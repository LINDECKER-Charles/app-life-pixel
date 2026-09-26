//! What the hosted MCP endpoint (A3) decides beyond the tools themselves: the plan's daily ceiling
//! of tool calls, and exports delivered as short-lived signed links, compiled on demand.
//!
//! [`DailyCeiling`] counts an account's calls per UTC day through [`McpUsageStore`];
//! [`ExportLinks`] signs and checks links with `LP_EXPORT_LINK_SECRET`; [`ExportDownloads`]
//! compiles the file a link names from the animation's version it names, and nothing is stored.
//! With the `testing` feature, [`memory`] holds in-memory adapters of the ports.
//!
//! [`McpUsageStore`]: ports::McpUsageStore

mod ceiling;
mod downloads;
mod error;
mod export_link;
pub mod ports;

#[cfg(feature = "testing")]
pub mod memory;
#[cfg(test)]
mod tests;

use std::sync::Arc;

pub use ceiling::DailyCeiling;
pub use downloads::ExportDownloads;
pub use error::McpError;
pub use export_link::{EXPORT_LINK_LIFETIME, ExportLink, ExportLinks};

use self::ports::{AnimationOwners, McpUsageStore};

/// The stores of the hosted MCP endpoint.
#[derive(Clone)]
pub struct McpStores {
    /// The calls of each account per UTC day.
    pub usage: Arc<dyn McpUsageStore>,
    /// Whose animation a link names.
    pub owners: Arc<dyn AnimationOwners>,
}
