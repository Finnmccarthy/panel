use clap::Args;

#[derive(Args)]
pub struct FinishArgs;

pub struct FinishCommand;

impl shared::extensions::commands::CliCommand<FinishArgs> for FinishCommand {
    fn get_command(&self, command: clap::Command) -> clap::Command {
        command
    }

    fn get_executor(self) -> Box<shared::extensions::commands::ExecutorFunc> {
        Box::new(|env, _arg_matches| {
            Box::pin(async move {
                let state = shared::AppState::new_cli(env).await?;

                state.settings.set_oobe_step(None).await?;

                eprintln!("oobe has been marked as finished");
                eprintln!("a running panel caches settings for up to 60 seconds");

                Ok(0)
            })
        })
    }
}
