use shared::extensions::commands::CliCommandGroupBuilder;

mod pterodactyl_0x7d8;
mod pterodactyl_arix;

pub fn commands(cli: CliCommandGroupBuilder) -> CliCommandGroupBuilder {
    cli.add_command(
        "pterodactyl-0x7d8",
        "Imports 0x7d8 subdomain manager data into the Panel.",
        pterodactyl_0x7d8::Pterodactyl0x7d8Command,
    )
    .add_command(
        "pterodactyl-arix",
        "Imports Arix Addon Pack subdomain manager data into the Panel.",
        pterodactyl_arix::PterodactylArixCommand,
    )
}
