//! `life-pixel-server`: `serve` (the default), `openapi`, `admin-openapi`, `migrate` and
//! `healthcheck`; with the `stack-tests` feature, `test-database`.

use std::process::ExitCode;

use anyhow::Context;
use clap::{Parser, Subcommand};
use life_pixel_server::commands::{healthcheck, serve};
use life_pixel_server::config::{self, Config};
use life_pixel_server::telemetry::Telemetry;
use life_pixel_server::{admin, database, openapi};

/// The Life Pixel server.
#[derive(Parser)]
#[command(name = "life-pixel-server", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Subcommand)]
enum Command {
    /// Runs the migrations, then listens: the default.
    Serve,
    /// Prints the public API's description, as crates/server/openapi.json.
    Openapi,
    /// Prints the internal admin API's description, as crates/server/admin-openapi.json.
    AdminOpenapi,
    /// Runs the migrations, then exits.
    Migrate,
    /// Requests /healthz on the local listener: exits 0 when it answers 200, 1 otherwise.
    Healthcheck,
    /// Creates or drops a migrated database of the local stack, for end-to-end tests.
    #[cfg(feature = "stack-tests")]
    #[command(subcommand)]
    TestDatabase(TestDatabaseCommand),
}

/// `test-database create`, `test-database drop <url>`.
#[cfg(feature = "stack-tests")]
#[derive(Clone, Subcommand)]
enum TestDatabaseCommand {
    /// Creates and migrates a test database, and prints its URL.
    Create,
    /// Drops the test database of `url`.
    Drop {
        /// The URL `test-database create` printed.
        url: String,
    },
}

fn main() -> anyhow::Result<ExitCode> {
    match Cli::parse().command.unwrap_or(Command::Serve) {
        Command::Openapi => print_openapi(),
        Command::AdminOpenapi => print_admin_openapi(),
        Command::Healthcheck => check_health(),
        #[cfg(feature = "stack-tests")]
        Command::TestDatabase(command) => test_database(&command),
        command @ (Command::Serve | Command::Migrate) => run(&command),
    }
}

fn print_openapi() -> anyhow::Result<ExitCode> {
    print!("{}", openapi::to_json()?);
    Ok(ExitCode::SUCCESS)
}

fn print_admin_openapi() -> anyhow::Result<ExitCode> {
    print!("{}", admin::openapi::to_json()?);
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
fn run(command: &Command) -> anyhow::Result<ExitCode> {
    config::load_development_env().context("cannot read .env")?;
    let config = Config::from_env()?;
    let telemetry = Telemetry::init(&config)?;
    let result = runtime()?.block_on(async move {
        if matches!(command, Command::Migrate) {
            let pool = database::connect(&config.database).await?;
            return database::migrate(&pool).await.map_err(anyhow::Error::from);
        }
        serve::serve(config).await.map_err(anyhow::Error::from)
    });
    if let Err(error) = &result {
        tracing::error!(error = %error, "the server stopped on an error");
    }
    telemetry.shutdown();
    result.map(|()| ExitCode::SUCCESS)
}

/// Creates a test database and prints its URL, or drops the one of a URL.
#[cfg(feature = "stack-tests")]
fn test_database(command: &TestDatabaseCommand) -> anyhow::Result<ExitCode> {
    use life_pixel_server::testing::TestDatabase;
    config::load_development_env().context("cannot read .env")?;
    runtime()?.block_on(async {
        match command {
            TestDatabaseCommand::Create => println!("{}", TestDatabase::create().await?.keep()?),
            TestDatabaseCommand::Drop { url } => TestDatabase::drop_url(url).await?,
        }
        Ok(ExitCode::SUCCESS)
    })
}

fn runtime() -> std::io::Result<tokio::runtime::Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
}
