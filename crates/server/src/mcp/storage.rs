//! The MCP endpoint's stores on Postgres: the calls of each account per UTC day in `mcp_usage`,
//! and the owner of an animation a signed link names.

use std::sync::Arc;

use async_trait::async_trait;
use life_pixel_service::mcp::McpStores;
use life_pixel_service::mcp::ports::{AnimationOwners, McpStoreError, McpUsageStore};
use life_pixel_service::{AccountId, AnimationId};
use sqlx::PgPool;
use time::Date;
use uuid::Uuid;

use crate::storage::metrics::timed;

/// Adds one call, unless the day's count already reached the limit: no row comes back then.
const ADD_CALL: &str = "insert into mcp_usage (account_id, day, calls) values ($1, $2, 1) \
                        on conflict (account_id, day) do update set calls = mcp_usage.calls + 1 \
                        where mcp_usage.calls < $3 returning calls";
/// The owner of an animation, while the account is active.
const OWNER: &str = "select n.account_id from animations n join accounts a on a.id = n.account_id \
                     where n.id = $1 and a.status = 'active'";

/// The MCP stores over the migrated database of `pool`.
#[must_use]
pub fn hosted_stores(pool: &PgPool) -> McpStores {
    McpStores {
        usage: Arc::new(PostgresMcpUsage::new(pool.clone())),
        owners: Arc::new(PostgresAnimationOwners::new(pool.clone())),
    }
}

/// The `mcp_usage` table: one row per account and UTC day.
#[derive(Clone)]
pub struct PostgresMcpUsage {
    pool: PgPool,
}

impl PostgresMcpUsage {
    /// The store over the migrated database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl McpUsageStore for PostgresMcpUsage {
    async fn add_call(
        &self,
        account: AccountId,
        (day, limit): (Date, u32),
    ) -> Result<bool, McpStoreError> {
        if limit == 0 {
            return Ok(false);
        }
        let limit = i32::try_from(limit).unwrap_or(i32::MAX);
        let query = sqlx::query_scalar::<_, i32>(ADD_CALL)
            .bind(account.uuid())
            .bind(day)
            .bind(limit);
        let added = timed("add_mcp_call", query.fetch_optional(&self.pool)).await;
        Ok(added.map_err(database)?.is_some())
    }
}

/// The animations' owners, from the `animations` table.
#[derive(Clone)]
pub struct PostgresAnimationOwners {
    pool: PgPool,
}

impl PostgresAnimationOwners {
    /// The store over the migrated database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AnimationOwners for PostgresAnimationOwners {
    async fn owner_of(&self, id: AnimationId) -> Result<Option<AccountId>, McpStoreError> {
        let query = sqlx::query_scalar::<_, Uuid>(OWNER).bind(id.uuid());
        let owner = timed("select_animation_owner", query.fetch_optional(&self.pool)).await;
        Ok(owner.map_err(database)?.map(AccountId::from_uuid))
    }
}

/// The failure of a query.
fn database(error: sqlx::Error) -> McpStoreError {
    McpStoreError(format!("database: {error}"))
}
