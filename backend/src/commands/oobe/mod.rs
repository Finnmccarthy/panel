use shared::extensions::commands::CliCommandGroupBuilder;

mod finish;
mod reset;

pub fn commands(cli: CliCommandGroupBuilder) -> CliCommandGroupBuilder {
    cli.add_command(
        "reset",
        "Resets the OOBE to a given step.",
        reset::ResetCommand,
    )
    .add_command(
        "finish",
        "Marks the OOBE as finished.",
        finish::FinishCommand,
    )
}
