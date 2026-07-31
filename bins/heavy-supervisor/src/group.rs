use anyhow::Context;
use rustix::process::{Pid, Signal, WaitOptions};
use std::time::Duration;

pub const REAP_BOUND: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Group {
    pgid: Pid,
}

impl Group {
    pub fn of(child: &tokio::process::Child) -> anyhow::Result<Self> {
        let pgid = child
            .id()
            .and_then(|raw| i32::try_from(raw).ok())
            .and_then(Pid::from_raw)
            .context("the child reported no usable pid")?;

        Ok(Self { pgid })
    }

    pub fn signal(&self, signal: Signal) -> anyhow::Result<()> {
        match rustix::process::kill_process_group(self.pgid, signal) {
            Ok(()) | Err(rustix::io::Errno::SRCH) => Ok(()),
            Err(err) => {
                Err(err).with_context(|| format!("signaling the process group {:?}", self.pgid))
            }
        }
    }

    pub fn sweep(&self) {
        let _ = self.signal(Signal::KILL);

        while let Ok(Some(_)) = rustix::process::waitpgid(self.pgid, WaitOptions::NOHANG) {}
    }
}
