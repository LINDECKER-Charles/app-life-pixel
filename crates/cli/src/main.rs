//! `life-pixel`: `mcp`, `list`, `export`, `--version` — `docs/v1/mcp-cli.md`'s "A4 — CLI".
//!
//! Logs always go to `tracing` on stderr; `println!` carries the protocol in `mcp` mode and the
//! commands' own results elsewhere. A failure prints its message, in French when `LC_ALL`,
//! `LC_MESSAGES` or `LANG` starts with `fr`, in English otherwise, and exits 1; success exits 0.

use std::process::ExitCode;

use clap::Parser;
use life_pixel_cli::args::{Cli, Command};
use life_pixel_cli::messages::{Language, Messages};
use life_pixel_cli::{AppError, commands};

fn main() -> ExitCode {
    init_logging();
    let messages = Messages::load(Language::from_env());
    match runtime().and_then(|runtime| runtime.block_on(dispatch(Cli::parse().command, &messages)))
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{}", error.message(&messages));
            ExitCode::FAILURE
        }
    }
}

/// Runs `command`, `list` and `export` speaking `messages`; `mcp` never prints through them —
/// the protocol carries its own failures.
async fn dispatch(command: Command, messages: &Messages) -> Result<(), AppError> {
    match command {
        Command::Mcp(args) => commands::mcp::run(args).await,
        Command::List(args) => commands::list::run(args, messages).await,
        Command::Export(args) => commands::export::run(args, messages).await,
    }
}

/// The single-process runtime every command runs on.
fn runtime() -> Result<tokio::runtime::Runtime, AppError> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| AppError::Plain(error.to_string()))
}

/// Structured logs on stderr: stdout stays free for `mcp`'s protocol and the other commands'
/// results.
fn init_logging() {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_target(false)
        .init();
}
