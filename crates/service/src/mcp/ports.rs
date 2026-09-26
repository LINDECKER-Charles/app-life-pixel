//! The ports of the hosted MCP endpoint: the daily count of calls, and the owners of animations.

use async_trait::async_trait;
use thiserror::Error;
use time::Date;

use crate::ids::{AccountId, AnimationId};

/// Why a store of the MCP endpoint failed: the detail is for the logs.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("mcp storage unavailable: {0}")]
pub struct McpStoreError(pub String);

/// The tool calls of each account per UTC day.
#[async_trait]
pub trait McpUsageStore: Send + Sync {
    /// Counts one call of `account` on `day`, unless it already made `limit`: whether it was
    /// counted. Checking and counting are one change.
    async fn add_call(
        &self,
        account: AccountId,
        (day, limit): (Date, u32),
    ) -> Result<bool, McpStoreError>;
}

/// Whose animation a signed link names: the link carries no account.
#[async_trait]
pub trait AnimationOwners: Send + Sync {
    /// The account the animation `id` belongs to, when it exists and its account may act.
    async fn owner_of(&self, id: AnimationId) -> Result<Option<AccountId>, McpStoreError>;
}
