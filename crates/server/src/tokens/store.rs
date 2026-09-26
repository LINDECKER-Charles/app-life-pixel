//! `PostgresTokenStore`: the `access_tokens` table. Counting an account's active tokens and
//! adding one happen under a lock of the account's row, so that two creations never both pass
//! the limit.

use async_trait::async_trait;
use life_pixel_service::AccountId;
use life_pixel_service::accounts::TokenHash;
use life_pixel_service::accounts::ports::AccountStatus;
use life_pixel_service::tokens::TokenScope;
use life_pixel_service::tokens::ports::{
    AccessTokenRecord, FoundToken, TokenStore, TokenStoreError,
};
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::storage::metrics::timed;

/// The columns of a token's row, in [`TokenRow`]'s order.
macro_rules! token_columns {
    () => {
        "t.id, t.account_id, t.name, t.prefix, t.scopes, t.created_at, t.expires_at, \
         t.last_used_at, t.revoked_at"
    };
}

const LOCK_ACCOUNT: &str = "select id from accounts where id = $1 for update";
const COUNT_ACTIVE: &str = "select count(*) from access_tokens \
                            where account_id = $1 and revoked_at is null and expires_at > $2";
const INSERT: &str = "insert into access_tokens (id, account_id, name, token_hash, prefix, \
                      scopes, created_at, expires_at) values ($1, $2, $3, $4, $5, $6, $7, $8)";
const LIST: &str = concat!(
    "select ",
    token_columns!(),
    " from access_tokens t where t.account_id = $1 \
     and ($2::timestamptz is null or (t.revoked_at is null and t.expires_at > $2)) \
     order by t.created_at desc, t.id desc"
);
const REVOKE: &str = "update access_tokens set revoked_at = $3 \
                      where id = $2 and account_id = $1 and revoked_at is null and expires_at > $3";
const FIND: &str = concat!(
    "select ",
    token_columns!(),
    ", a.status from access_tokens t join accounts a on a.id = t.account_id \
     where t.token_hash = $1"
);
const TOUCH: &str = "update access_tokens set last_used_at = $2 where id = $1";

/// A token's row: never its hash.
#[derive(sqlx::FromRow)]
struct TokenRow {
    id: Uuid,
    account_id: Uuid,
    name: String,
    prefix: String,
    scopes: Vec<String>,
    created_at: OffsetDateTime,
    expires_at: OffsetDateTime,
    last_used_at: Option<OffsetDateTime>,
    revoked_at: Option<OffsetDateTime>,
}

impl TokenRow {
    fn into_record(self) -> Result<AccessTokenRecord, TokenStoreError> {
        let scopes = self.scopes.iter().map(|scope| {
            TokenScope::parse(scope).ok_or_else(|| corrupt("access_tokens.scopes", scope))
        });
        Ok(AccessTokenRecord {
            id: self.id,
            account: AccountId::from_uuid(self.account_id),
            name: self.name,
            prefix: self.prefix,
            scopes: scopes.collect::<Result<_, _>>()?,
            created_at: self.created_at,
            expires_at: self.expires_at,
            last_used_at: self.last_used_at,
            revoked_at: self.revoked_at,
        })
    }
}

/// A token's row and its account's status.
#[derive(sqlx::FromRow)]
struct FoundRow {
    #[sqlx(flatten)]
    token: TokenRow,
    status: String,
}

/// The tokens of the migrated database of a pool.
#[derive(Clone)]
pub struct PostgresTokenStore {
    pool: PgPool,
}

impl PostgresTokenStore {
    /// The store over the migrated database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TokenStore for PostgresTokenStore {
    async fn create(
        &self,
        token: &AccessTokenRecord,
        (hash, max_active): (&TokenHash, usize),
    ) -> Result<(), TokenStoreError> {
        let mut transaction = self.pool.begin().await.map_err(database)?;
        let account = token.account.uuid();
        let lock = sqlx::query(LOCK_ACCOUNT).bind(account);
        timed("lock_token_account", lock.execute(&mut *transaction))
            .await
            .map_err(database)?;
        let count = sqlx::query_scalar::<_, i64>(COUNT_ACTIVE)
            .bind(account)
            .bind(token.created_at);
        let active = timed("count_active_tokens", count.fetch_one(&mut *transaction)).await;
        let active = usize::try_from(active.map_err(database)?).unwrap_or(usize::MAX);
        if active >= max_active {
            return Err(TokenStoreError::LimitReached);
        }
        let scopes: Vec<&str> = token.scopes.iter().map(|scope| scope.as_str()).collect();
        let insert = sqlx::query(INSERT)
            .bind(token.id)
            .bind(account)
            .bind(&token.name)
            .bind(hash.as_bytes().as_slice())
            .bind(&token.prefix)
            .bind(scopes)
            .bind(token.created_at)
            .bind(token.expires_at);
        timed("insert_access_token", insert.execute(&mut *transaction))
            .await
            .map_err(database)?;
        transaction.commit().await.map_err(database)
    }

    async fn list(
        &self,
        account: AccountId,
        active_at: Option<OffsetDateTime>,
    ) -> Result<Vec<AccessTokenRecord>, TokenStoreError> {
        let query = sqlx::query_as::<_, TokenRow>(LIST)
            .bind(account.uuid())
            .bind(active_at);
        let rows = timed("list_access_tokens", query.fetch_all(&self.pool)).await;
        rows.map_err(database)?
            .into_iter()
            .map(TokenRow::into_record)
            .collect()
    }

    async fn revoke(
        &self,
        (account, id): (AccountId, Uuid),
        at: OffsetDateTime,
    ) -> Result<bool, TokenStoreError> {
        let query = sqlx::query(REVOKE).bind(account.uuid()).bind(id).bind(at);
        let revoked = timed("revoke_access_token", query.execute(&self.pool)).await;
        Ok(revoked.map_err(database)?.rows_affected() > 0)
    }

    async fn find(&self, hash: &TokenHash) -> Result<Option<FoundToken>, TokenStoreError> {
        let query = sqlx::query_as::<_, FoundRow>(FIND).bind(hash.as_bytes().as_slice());
        let row = timed("select_access_token", query.fetch_optional(&self.pool)).await;
        let Some(row) = row.map_err(database)? else {
            return Ok(None);
        };
        let account_status = AccountStatus::parse(&row.status)
            .ok_or_else(|| corrupt("accounts.status", &row.status))?;
        Ok(Some(FoundToken {
            record: row.token.into_record()?,
            account_status,
        }))
    }

    async fn touch(&self, id: Uuid, at: OffsetDateTime) -> Result<(), TokenStoreError> {
        let query = sqlx::query(TOUCH).bind(id).bind(at);
        timed("touch_access_token", query.execute(&self.pool))
            .await
            .map_err(database)?;
        Ok(())
    }
}

/// The failure of a query.
fn database(error: sqlx::Error) -> TokenStoreError {
    TokenStoreError::Unavailable(format!("database: {error}"))
}

/// A value the schema should have prevented.
fn corrupt(column: &str, value: &str) -> TokenStoreError {
    TokenStoreError::Unavailable(format!("{column} holds an unexpected value: {value}"))
}
