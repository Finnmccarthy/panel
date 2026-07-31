use super::{StepCommand, StepOutput};
use crate::group::Group;
use anyhow::Context;
use rustix::process::Signal;
use std::{
    process::{ExitStatus, Stdio},
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

const NOTE_PREFIX: &str = "[supervisor] ";

const CANCEL_GRACE: Duration = Duration::from_secs(1);

pub trait CommandRunner {
    fn run(
        &mut self,
        command: &StepCommand,
    ) -> impl std::future::Future<Output = anyhow::Result<StepOutput>> + Send;

    fn note(
        &mut self,
        message: &str,
    ) -> impl std::future::Future<Output = anyhow::Result<()>> + Send;
}

#[derive(Debug)]
pub struct Cancellation {
    cancelled: tokio::sync::watch::Sender<bool>,
    step: Mutex<Option<Group>>,
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::new()
    }
}

impl Cancellation {
    pub fn new() -> Self {
        Self {
            cancelled: tokio::sync::watch::channel(false).0,
            step: Mutex::new(None),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        *self.cancelled.borrow()
    }

    pub fn cancel(&self) {
        let step = self.armed();
        self.cancelled.send_replace(true);

        if let Some(group) = *step
            && let Err(err) = group.signal(Signal::TERM)
        {
            tracing::warn!("{err:#}");
        }
    }

    async fn requested(&self) {
        let mut cancelled = self.cancelled.subscribe();
        let _ = cancelled.wait_for(|cancelled| *cancelled).await;
    }

    fn armed(&self) -> MutexGuard<'_, Option<Group>> {
        self.step.lock().unwrap_or_else(|err| err.into_inner())
    }
}

struct Step<'a> {
    cancel: &'a Cancellation,
    group: Group,
}

impl Drop for Step<'_> {
    fn drop(&mut self) {
        *self.cancel.armed() = None;
        let _ = self.group.signal(Signal::KILL);
    }
}

enum Interrupt {
    TimedOut,
    Cancelled,
}

enum Ended {
    Finished(std::io::Result<ExitStatus>),
    Abandoned { interrupt: Interrupt, settled: bool },
}

pub struct ProcessRunner {
    log: tokio::fs::File,
    cancel: Arc<Cancellation>,
}

impl ProcessRunner {
    pub fn new(log: std::fs::File, cancel: Arc<Cancellation>) -> Self {
        Self {
            log: tokio::fs::File::from_std(log),
            cancel,
        }
    }
}

impl CommandRunner for ProcessRunner {
    async fn run(&mut self, command: &StepCommand) -> anyhow::Result<StepOutput> {
        let description = command.describe();
        let cancel = Arc::clone(&self.cancel);

        if cancel.is_cancelled() {
            anyhow::bail!("{description} was not started, the build was cancelled");
        }

        let mut child = tokio::process::Command::new(&command.program)
            .args(&command.args)
            .current_dir(&command.cwd)
            .envs(command.env.iter().map(|(key, value)| (key, value)))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .kill_on_drop(true)
            .spawn()
            .with_context(|| format!("running {description}"))?;

        let group = match Group::of(&child) {
            Ok(group) => group,
            Err(err) => {
                let _ = child.start_kill();

                return Err(err).with_context(|| format!("running {description}"));
            }
        };
        let step = Step {
            cancel: &cancel,
            group,
        };
        *step.cancel.armed() = Some(group);

        let stdout = child.stdout.take().context("child stdout was not piped")?;
        let stderr = child.stderr.take().context("child stderr was not piped")?;

        let (lines, mut receiver) = tokio::sync::mpsc::channel(64);
        let stderr_lines = lines.clone();
        let pumps = [
            tokio::spawn(async move { pump(stdout, true, lines).await }),
            tokio::spawn(async move { pump(stderr, false, stderr_lines).await }),
        ];

        let mut captured = String::new();
        let ended = {
            let mut drained = std::pin::pin!(drain(
                &mut self.log,
                &mut receiver,
                &mut captured,
                command.capture_stdout,
                &mut child,
            ));

            let interrupted = tokio::select! {
                finished = &mut drained => Ok(finished),
                () = elapsed(command.timeout) => Err(Interrupt::TimedOut),
                () = cancel.requested() => Err(Interrupt::Cancelled),
            };

            match interrupted {
                Ok(finished) => Ended::Finished(finished),
                Err(interrupt) => {
                    if let Err(err) = group.signal(Signal::TERM) {
                        tracing::warn!("{err:#}");
                    }

                    let mut settled = tokio::time::timeout(CANCEL_GRACE, &mut drained)
                        .await
                        .is_ok();
                    if !settled {
                        if let Err(err) = group.signal(Signal::KILL) {
                            tracing::warn!("{err:#}");
                        }
                        settled = tokio::time::timeout(crate::group::REAP_BOUND, &mut drained)
                            .await
                            .is_ok();
                    }

                    if settled {
                        group.sweep();
                    }

                    Ended::Abandoned { interrupt, settled }
                }
            }
        };

        let status = match ended {
            Ended::Finished(status) => {
                status.with_context(|| format!("waiting for {description}"))?
            }
            Ended::Abandoned { interrupt, settled } => {
                if !settled {
                    for pump in pumps {
                        pump.abort();
                    }

                    while let Ok((from_stdout, line)) = receiver.try_recv() {
                        if from_stdout && command.capture_stdout {
                            continue;
                        }

                        let _ = self.log.write_all(line.as_bytes()).await;
                        let _ = self.log.write_all(b"\n").await;
                    }
                }

                let _ = self.log.flush().await;

                return Err(match interrupt {
                    Interrupt::TimedOut => anyhow::anyhow!(
                        "{description} did not finish within {} seconds",
                        command.timeout.unwrap_or_default().as_secs_f64()
                    ),
                    Interrupt::Cancelled => anyhow::anyhow!("{description} was stopped"),
                });
            }
        };

        Ok(StepOutput {
            exit_code: status.code(),
            stdout: captured,
        })
    }

    async fn note(&mut self, message: &str) -> anyhow::Result<()> {
        self.log.write_all(NOTE_PREFIX.as_bytes()).await?;
        self.log.write_all(message.as_bytes()).await?;
        self.log.write_all(b"\n").await?;

        self.log.flush().await.context("writing to the build log")
    }
}

async fn drain(
    log: &mut tokio::fs::File,
    receiver: &mut tokio::sync::mpsc::Receiver<(bool, String)>,
    captured: &mut String,
    capture_stdout: bool,
    child: &mut tokio::process::Child,
) -> std::io::Result<ExitStatus> {
    while let Some((from_stdout, line)) = receiver.recv().await {
        if from_stdout && capture_stdout {
            captured.push_str(&line);
            captured.push('\n');

            continue;
        }

        log.write_all(line.as_bytes()).await?;
        log.write_all(b"\n").await?;

        if let Some(stage) = super::classify_stage(&line) {
            tracing::info!("build reached {stage:?}");
        }
    }

    log.flush().await?;

    child.wait().await
}

async fn elapsed(timeout: Option<Duration>) {
    match timeout {
        Some(timeout) => tokio::time::sleep(timeout).await,
        None => std::future::pending().await,
    }
}

async fn pump(
    pipe: impl tokio::io::AsyncRead + Unpin,
    from_stdout: bool,
    lines: tokio::sync::mpsc::Sender<(bool, String)>,
) {
    let mut reader = tokio::io::BufReader::new(pipe);
    let mut line = Vec::new();

    loop {
        line.clear();
        match reader.read_until(b'\n', &mut line).await {
            Ok(0) => return,
            Ok(_) => {}
            Err(err) => {
                let _ = lines
                    .send((
                        false,
                        format!("{NOTE_PREFIX}reading the step's output failed: {err}"),
                    ))
                    .await;

                return;
            }
        }

        let text = String::from_utf8_lossy(&line)
            .trim_end_matches('\n')
            .trim_end_matches('\r')
            .to_string();

        if lines.send((from_stdout, text)).await.is_err() {
            return;
        }
    }
}
