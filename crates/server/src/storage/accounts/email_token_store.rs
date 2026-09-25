//! `PostgresEmailTokenStore`: the `email_tokens` table. A token is spent by one `update` that
//! matches it only while it is unused and unexpired, so that of two uses one at most succeeds.

use async_trait::async_trait;
use life_pixel_service::AccountId;
use life_pixel_service::accounts::ports::{
    AccountStoreError, EmailTokenRecord, EmailTokenStore, TokenUse,
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::rows::database;
use crate::storage::metrics::timed;

const INSERT: &str = "insert into email_tokens (token_hash, account_id, purpose, created_at, \
                      expires_at) values ($1, $2, $3, $4, $5)";
const CONSUME: &str = "update email_tokens set used_at = $3 \
                       where token_hash = $1 and purpose = $2 and used_at is null \
                       and expires_at > $3 returning account_id";
const PURGE: &str = "delete from email_tokens where expires_at <= $1";

/// The emailed tokens of the migrated database of a pool.
#[derive(Clone)]
pub struct PostgresEmailTokenStore {
    pool: PgPool,
}

impl PostgresEmailTokenStore {
    /// The store over the migrated database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl EmailTokenStore for PostgresEmailTokenStore {
    async fn create(&self, token: EmailTokenRecord) -> Result<(), AccountStoreError> {
        let query = sqlx::query(INSERT)
            .bind(token.token_hash.as_bytes().as_slice())
            .bind(token.account_id.uuid())
            .bind(token.purpose.as_str())
            .bind(token.created_at)
            .bind(token.expires_at);
        timed("insert_email_token", query.execute(&self.pool))
            .await
            .map_err(database)?;
        Ok(())
    }

    async fn consume(&self, used: TokenUse) -> Result<Option<AccountId>, AccountStoreError> {
        let query = sqlx::query_scalar::<_, Uuid>(CONSUME)
            .bind(used.token_hash.as_bytes().as_slice())
            .bind(used.purpose.as_str())
            .bind(used.at);
        let account = timed("consume_email_token", query.fetch_optional(&self.pool)).await;
        Ok(account.map_err(database)?.map(AccountId::from_uuid))
    }

    async fn purge_expired(&self, now: OffsetDateTime) -> Result<u64, AccountStoreError> {
        let query = sqlx::query(PURGE).bind(now);
        let purged = timed("purge_email_tokens", query.execute(&self.pool)).await;
        Ok(purged.map_err(database)?.rows_affected())
    }
}
