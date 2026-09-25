//! `life-pixel-server`: `serve` (the default), `openapi`, `migrate` and `healthcheck`.

use std::process::ExitCode;

use anyhow::Context;
use clap::{Parser, Subcommand};
use life_pixel_server::commands::{healthcheck, serve};
use life_pixel_server::config::{self, Config};
use life_pixel_server::telemetry::Telemetry;
use life_pixel_server::{database, openapi};

/// The Life Pixel server.
#[derive(Parser)]
#[command(name = "life-pixel-server", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Copy, Subcommand)]
enum Command {
    /// Runs the migrations, then listens: the default.
    Serve,
    /// Prints the public API's description, as crates/server/openapi.json.
    Openapi,
    /// Runs the migrations, then exits.
    Migrate,
    /// Requests /healthz on the local listener: exits 0 when it answers 200, 1 otherwise.
    Healthcheck,
}

fn main() -> anyhow::Result<ExitCode> {
    match Cli::parse().command.unwrap_or(Command::Serve) {
        Command::Openapi => print_openapi(),
        Command::Healthcheck => check_health(),
        command @ (Command::Serve | Command::Migrate) => run(command),
    }
}

fn print_openapi() -> anyhow::Result<ExitCode> {
    print!("{}", openapi::to_json()?);
    Ok(ExitCode::SUCCESS)
}

fn check_health() -> anyhow::Result<ExitCode> {
    config::load_development_env().context("cannot read .env")?;
    let address = config::http_addr_from_env()?;
    let healthy = runtime()?.block_on(healthcheck::healthcheck(address));
    Ok(if healthy {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

/// `serve` or `migrate`, with telemetry; the traces still buffered are sent before exiting.
fn run(command: Command) -> anyhow::Result<ExitCode> {
    config::load_development_env().context("cannot read .env")?;
    let config = Config::from_env()?;
    let telemetry = Telemetry::init(&config)?;
    let result = runtime()?.block_on(async move {
        if matches!(command, Command::Migrate) {
            return database::migrate(&config.database)
                .await
                .map_err(anyhow::Error::from);
        }
        serve::serve(config).await.map_err(anyhow::Error::from)
    });
    if let Err(error) = &result {
        tracing::error!(error = %error, "the server stopped on an error");
    }
    telemetry.shutdown();
    result.map(|()| ExitCode::SUCCESS)
}

fn runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
}
