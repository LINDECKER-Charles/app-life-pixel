//! Why the hosted MCP endpoint refused, as a code the interface and the agents read.

use serde_json::{Map, Value, json};
use thiserror::Error;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use super::ports::McpStoreError;
use crate::error::{Coded, CodedError, params};

/// The error of the daily ceiling or of a signed link.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum McpError {
    /// `mcp.daily_limit`: the account made its plan's calls for the UTC day.
    #[error("daily limit of {limit} calls reached")]
    DailyLimit {
        /// The plan's calls per day.
        limit: u32,
        /// When the count starts again: the next midnight, UTC.
        resets_at: OffsetDateTime,
    },
    /// `export.link_invalid`: tampered with, expired, or naming a version that changed since.
    #[error("invalid export link")]
    LinkInvalid,
    /// `service.unavailable`: the storage failed; the detail is logged.
    #[error("service unavailable")]
    Unavailable,
}

impl From<McpStoreError> for McpError {
    fn from(error: McpStoreError) -> Self {
        tracing::error!(detail = %error.0, "mcp storage unavailable");
        Self::Unavailable
    }
}

impl Coded for McpError {
    fn code(&self) -> &'static str {
        match self {
            Self::DailyLimit { .. } => "mcp.daily_limit",
            Self::LinkInvalid => "export.link_invalid",
            Self::Unavailable => "service.unavailable",
        }
    }

    fn params(&self) -> Map<String, Value> {
        let Self::DailyLimit { limit, resets_at } = self else {
            return Map::new();
        };
        let resets_at = resets_at.format(&Rfc3339).unwrap_or_default();
        params([("limit", json!(limit)), ("resetsAt", json!(resets_at))])
    }
}

impl From<McpError> for CodedError {
    fn from(error: McpError) -> Self {
        Self::of(&error)
    }
}
