//! `cargo xtask <command>`: the build commands Cargo cannot express. Each command is a file of
//! `commands/`; `cargo xtask --help` lists them.

mod boundaries;
mod commands;

use clap::Parser;

/// The repository's build commands.
#[derive(Parser)]
#[command(bin_name = "cargo xtask")]
struct Cli {
    #[command(subcommand)]
    command: commands::Command,
}

fn main() -> anyhow::Result<()> {
    Cli::parse().command.run()
}
