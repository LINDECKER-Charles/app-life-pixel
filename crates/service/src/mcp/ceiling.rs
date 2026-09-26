//! The plan's daily ceiling of MCP tool calls, per account and UTC day.

use std::sync::Arc;

use time::{Duration, OffsetDateTime, Time, UtcOffset};

use super::McpError;
use super::ports::McpUsageStore;
use crate::ids::AccountId;
use crate::plans::Plans;
use crate::ports::Clock;

/// Counts each tool call against `LP_PLAN_FREE_MCP_CALLS_PER_DAY`.
#[derive(Clone)]
pub struct DailyCeiling {
    usage: Arc<dyn McpUsageStore>,
    clock: Arc<dyn Clock>,
    plans: Plans,
}

impl DailyCeiling {
    /// The ceiling of `plans`, counted in `usage`, at the time of `clock`.
    #[must_use]
    pub fn new(usage: Arc<dyn McpUsageStore>, clock: Arc<dyn Clock>, plans: Plans) -> Self {
        Self {
            usage,
            clock,
            plans,
        }
    }

    /// Counts one tool call of `account` for the current UTC day.
    ///
    /// # Errors
    ///
    /// `mcp.daily_limit`, with `limit` and `resetsAt`, once the day's calls are made — the call
    /// is then not counted —; `service.unavailable`.
    pub async fn count_call(&self, account: AccountId) -> Result<(), McpError> {
        let now = self.clock.now().to_offset(UtcOffset::UTC);
        let limit = self.plans.free_mcp_calls_per_day;
        if self.usage.add_call(account, (now.date(), limit)).await? {
            return Ok(());
        }
        Err(McpError::DailyLimit {
            limit,
            resets_at: next_midnight(now),
        })
    }
}

/// The midnight, UTC, that ends the day of `now`.
fn next_midnight(now: OffsetDateTime) -> OffsetDateTime {
    now.replace_time(Time::MIDNIGHT) + Duration::days(1)
}
