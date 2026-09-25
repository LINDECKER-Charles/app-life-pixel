//! The plan values, read from configuration: no quota is written in code.

/// What the free plan allows. The hosted server reads it from its configuration; the local
/// library has no plan.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Plans {
    /// The storage quota of a free account, in bytes.
    pub free_storage_bytes: u64,
    /// The MCP calls a free account may make per day.
    pub free_mcp_calls_per_day: u32,
}
