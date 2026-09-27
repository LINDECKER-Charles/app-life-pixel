//! `life-pixel-admin-server`: `serve` (the default), `openapi`, `migrate`, `healthcheck`,
//! `create-admin` and `disable-admin`; with the `stack-tests` feature, `test-database`.

use std::process::ExitCode;

use anyhow::Context;
use clap::{Parser, Subcommand};
use life_pixel_admin_server::commands::admins::{self, PasswordInput};
use life_pixel_admin_server::commands::{healthcheck, serve};
use life_pixel_admin_server::config::{self, AccountsConfig, Config};
use life_pixel_admin_server::telemetry::Telemetry;
use life_pixel_admin_server::{database, openapi};

/// The Life Pixel admin server.
#[derive(Parser)]
#[command(name = "life-pixel-admin-server", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Clone, Subcommand)]
enum Command {
    /// Runs the migrations, then listens: the default.
    Serve,
    /// Prints the console's API description, as crates/admin-server/openapi.json.
    Openapi,
    /// Runs the migrations, then exits.
    Migrate,
    /// Requests /healthz on the local listener: exits 0 when it answers 200, 1 otherwise.
    Healthcheck,
    /// Creates an admin, and prints the otpauth:// URI of its TOTP secret.
    CreateAdmin {
        /// The admin's email address.
        email: String,
        /// Reads the password from the first line of standard input, instead of asking twice.
        #[arg(long)]
        password_stdin: bool,
    },
    /// Disables an admin and ends its sessions.
    DisableAdmin {
        /// The admin's email address.
        email: String,
    },
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
        Command::Healthcheck => check_health(),
        Command::CreateAdmin {
            email,
            password_stdin,
        } => create_admin(&email, password_stdin),
        Command::DisableAdmin { email } => disable_admin(&email),
        #[cfg(feature = "stack-tests")]
        Command::TestDatabase(command) => test_database(&command),
        command @ (Command::Serve | Command::Migrate) => run(&command),
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

/// `create-admin`: the URI on standard output, the rest on standard error.
fn create_admin(email: &str, password_stdin: bool) -> anyhow::Result<ExitCode> {
    config::load_development_env().context("cannot read .env")?;
    let config = AccountsConfig::from_env()?;
    let input = if password_stdin {
        PasswordInput::Stdin
    } else {
        PasswordInput::Prompt
    };
    let password = admins::read_password(input)?;
    let created = admins::create_admin(&config, email, &password);
    let (admin, uri) = runtime()?.block_on(created)?;
    eprintln!(
        "admin {} created: scan the URI below with an authenticator app",
        admin.id
    );
    println!("{uri}");
    Ok(ExitCode::SUCCESS)
}

fn disable_admin(email: &str) -> anyhow::Result<ExitCode> {
    config::load_development_env().context("cannot read .env")?;
    let config = AccountsConfig::from_env()?;
    if runtime()?.block_on(admins::disable_admin(&config, email))? {
        eprintln!("admin disabled, its sessions ended");
        return Ok(ExitCode::SUCCESS);
    }
    eprintln!("no active admin has this address");
    Ok(ExitCode::FAILURE)
}

/// `serve` or `migrate`, with telemetry; the traces still buffered are sent before exiting.
fn run(command: &Command) -> anyhow::Result<ExitCode> {
    config::load_development_env().context("cannot read .env")?;
    let config = Config::from_env()?;
    let telemetry = Telemetry::init(&config)?;
    let result = runtime()?.block_on(async move {
        if matches!(command, Command::Migrate) {
            let pool = database::connect(&config.database_url).await?;
            return database::migrate(&pool).await.map_err(anyhow::Error::from);
        }
        serve::serve(config).await.map_err(anyhow::Error::from)
    });
    if let Err(error) = &result {
        tracing::error!(error = %error, "the admin server stopped on an error");
    }
    telemetry.shutdown();
    result.map(|()| ExitCode::SUCCESS)
}

/// Creates a test database and prints its URL, or drops the one of a URL.
#[cfg(feature = "stack-tests")]
fn test_database(command: &TestDatabaseCommand) -> anyhow::Result<ExitCode> {
    use life_pixel_admin_server::testing::TestDatabase;
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
