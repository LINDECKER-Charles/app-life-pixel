//! `PostgresSessionStore`: the `sessions` table, by the SHA-256 of each cookie's token.

use async_trait::async_trait;
use life_pixel_service::AccountId;
use life_pixel_service::accounts::TokenHash;
use life_pixel_service::accounts::ports::{
    AccountStoreError, SessionExtension, SessionRecord, SessionStore,
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::rows::{database, token_hash};
use crate::storage::metrics::timed;

const INSERT: &str = "insert into sessions (token_hash, account_id, created_at, last_seen_at, \
                      expires_at) values ($1, $2, $3, $4, $5)";
const SELECT: &str = "select token_hash, account_id, created_at, last_seen_at, expires_at \
                      from sessions where token_hash = $1";
const EXTEND: &str = "update sessions set last_seen_at = $2, expires_at = $3 where token_hash = $1";
const DELETE: &str = "delete from sessions where token_hash = $1";
const DELETE_ALL: &str = "delete from sessions where account_id = $1";
const DELETE_OTHERS: &str = "delete from sessions where account_id = $1 and token_hash <> $2";
const PURGE: &str = "delete from sessions where expires_at <= $1";

/// A session's row.
#[derive(sqlx::FromRow)]
struct SessionRow {
    token_hash: Vec<u8>,
    account_id: Uuid,
    created_at: OffsetDateTime,
    last_seen_at: OffsetDateTime,
    expires_at: OffsetDateTime,
}

impl SessionRow {
    fn into_record(self) -> Result<SessionRecord, AccountStoreError> {
        Ok(SessionRecord {
            token_hash: token_hash(&self.token_hash)?,
            account_id: AccountId::from_uuid(self.account_id),
            created_at: self.created_at,
            last_seen_at: self.last_seen_at,
            expires_at: self.expires_at,
        })
    }
}

/// The sessions of the migrated database of a pool.
#[derive(Clone)]
pub struct PostgresSessionStore {
    pool: PgPool,
}

impl PostgresSessionStore {
    /// The store over the migrated database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionStore for PostgresSessionStore {
    async fn create(&self, session: SessionRecord) -> Result<(), AccountStoreError> {
        let query = sqlx::query(INSERT)
            .bind(session.token_hash.as_bytes().as_slice())
            .bind(session.account_id.uuid())
            .bind(session.created_at)
            .bind(session.last_seen_at)
            .bind(session.expires_at);
        timed("insert_session", query.execute(&self.pool))
            .await
            .map_err(database)?;
        Ok(())
    }

    async fn find(
        &self,
        token_hash: &TokenHash,
    ) -> Result<Option<SessionRecord>, AccountStoreError> {
        let query = sqlx::query_as::<_, SessionRow>(SELECT).bind(token_hash.as_bytes().as_slice());
        let row = timed("select_session", query.fetch_optional(&self.pool)).await;
        row.map_err(database)?
            .map(SessionRow::into_record)
            .transpose()
    }

    async fn extend(
        &self,
        token_hash: &TokenHash,
        extension: SessionExtension,
    ) -> Result<(), AccountStoreError> {
        let query = sqlx::query(EXTEND)
            .bind(token_hash.as_bytes().as_slice())
            .bind(extension.seen_at)
            .bind(extension.expires_at);
        timed("extend_session", query.execute(&self.pool))
            .await
            .map_err(database)?;
        Ok(())
    }

    async fn delete(&self, token_hash: &TokenHash) -> Result<(), AccountStoreError> {
        let query = sqlx::query(DELETE).bind(token_hash.as_bytes().as_slice());
        timed("delete_session", query.execute(&self.pool))
            .await
            .map_err(database)?;
        Ok(())
    }

    async fn delete_all(&self, account: AccountId) -> Result<(), AccountStoreError> {
        let query = sqlx::query(DELETE_ALL).bind(account.uuid());
        timed("delete_sessions", query.execute(&self.pool))
            .await
            .map_err(database)?;
        Ok(())
    }

    async fn delete_others(
        &self,
        account: AccountId,
        kept: &TokenHash,
    ) -> Result<(), AccountStoreError> {
        let query = sqlx::query(DELETE_OTHERS)
            .bind(account.uuid())
            .bind(kept.as_bytes().as_slice());
        let deleted = timed("delete_other_sessions", query.execute(&self.pool)).await;
        deleted.map_err(database)?;
        Ok(())
    }

    async fn purge_expired(&self, now: OffsetDateTime) -> Result<u64, AccountStoreError> {
        let query = sqlx::query(PURGE).bind(now);
        let purged = timed("purge_sessions", query.execute(&self.pool)).await;
        Ok(purged.map_err(database)?.rows_affected())
    }
}
