use shared::extensions::commands::CliCommandGroupBuilder;

mod import;

pub fn commands(cli: CliCommandGroupBuilder) -> CliCommandGroupBuilder {
    cli.add_group(
        "import",
        "Imports subdomain manager data into the Panel.",
        import::commands,
    )
}
