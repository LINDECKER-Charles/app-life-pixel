//! `mcp`: serves the tools of `docs/mcp.md` over stdio, on the local library —
//! `docs/mcp.md`'s "Where it runs", the desktop and CLI row.

use std::sync::Arc;

use life_pixel_mcp::LifePixelMcp;
use rmcp::ServiceExt as _;
use rmcp::transport::io::stdio;

use crate::args::McpArgs;
use crate::caller_policy::LocalCallerPolicy;
use crate::local_delivery::LocalExportDelivery;
use crate::{AppError, library};

/// The server's name, announced locally — `LP_MCP_SERVER_NAME` is the hosted transport's own.
const SERVER_NAME: &str = "life-pixel";

/// Runs `mcp`: serves until the client closes the pipe. Only the protocol reaches stdout; logs
/// go to stderr.
///
/// # Errors
///
/// The library cannot be opened, or the stdio transport fails to start or run.
pub async fn run(args: McpArgs) -> Result<(), AppError> {
    let path = library::resolve_path(args.library);
    let (library, editing) = library::open(&path)?;
    let working_directory = std::env::current_dir().map_err(plain)?;
    let delivery = Arc::new(LocalExportDelivery::new(working_directory, &args.allow_dir));
    let policy = Arc::new(LocalCallerPolicy);
    let server = LifePixelMcp::new(SERVER_NAME, (library, editing), (delivery, policy));
    let running = server.serve(stdio()).await.map_err(plain)?;
    running.waiting().await.map_err(plain)?;
    Ok(())
}

fn plain(error: impl std::fmt::Display) -> AppError {
    AppError::Plain(error.to_string())
}
