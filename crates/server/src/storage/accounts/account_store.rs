//! `PostgresAccountStore`: the identity columns of `accounts`. Addresses are `citext`, compared
//! whatever their case.

use async_trait::async_trait;
use life_pixel_service::AccountId;
use life_pixel_service::accounts::ports::{
    AccountRecord, AccountStore, AccountStoreError, NewAccount,
};
use life_pixel_service::accounts::{EmailAddress, Language, PasswordHash};
use sqlx::postgres::PgArguments;
use sqlx::query::Query;
use sqlx::{PgPool, Postgres};
use time::OffsetDateTime;

use super::rows::{AccountRow, account_columns, database};
use crate::storage::metrics::timed;

/// The unique constraint of the addresses.
const EMAIL_CONSTRAINT: &str = "accounts_email_key";

const INSERT: &str = concat!(
    "insert into accounts (id, email, password_hash, language, created_at) \
     values ($1, $2, $3, $4, $5) returning ",
    account_columns!()
);
const SELECT_BY_ID: &str = concat!(
    "select ",
    account_columns!(),
    " from accounts where id = $1"
);
const SELECT_BY_EMAIL: &str = concat!(
    "select ",
    account_columns!(),
    " from accounts where email = $1::citext"
);
const SET_PASSWORD_HASH: &str = "update accounts set password_hash = $2 where id = $1";
const MARK_VERIFIED: &str =
    "update accounts set email_verified_at = coalesce(email_verified_at, $2) where id = $1";
const SET_LANGUAGE: &str = "update accounts set language = $2 where id = $1";
const DELETE: &str = "delete from accounts where id = $1";

/// The accounts of the migrated database of a pool.
#[derive(Clone)]
pub struct PostgresAccountStore {
    pool: PgPool,
}

impl PostgresAccountStore {
    /// The store over the migrated database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountStore for PostgresAccountStore {
    async fn create(&self, account: NewAccount) -> Result<AccountRecord, AccountStoreError> {
        let query = sqlx::query_as::<_, AccountRow>(INSERT)
            .bind(account.id.uuid())
            .bind(account.email.as_str())
            .bind(account.password_hash.as_phc())
            .bind(account.language.as_str())
            .bind(account.created_at);
        let row = timed("insert_account", query.fetch_one(&self.pool)).await;
        row.map_err(taken_or_database)?.into_record()
    }

    async fn get(&self, id: AccountId) -> Result<Option<AccountRecord>, AccountStoreError> {
        let query = sqlx::query_as::<_, AccountRow>(SELECT_BY_ID).bind(id.uuid());
        let row = timed("select_account", query.fetch_optional(&self.pool)).await;
        row.map_err(database)?
            .map(AccountRow::into_record)
            .transpose()
    }

    async fn find_by_email(
        &self,
        email: &EmailAddress,
    ) -> Result<Option<AccountRecord>, AccountStoreError> {
        let query = sqlx::query_as::<_, AccountRow>(SELECT_BY_EMAIL).bind(email.as_str());
        let row = timed("select_account_by_email", query.fetch_optional(&self.pool)).await;
        row.map_err(database)?
            .map(AccountRow::into_record)
            .transpose()
    }

    async fn set_password_hash(
        &self,
        id: AccountId,
        hash: &PasswordHash,
    ) -> Result<(), AccountStoreError> {
        let query = sqlx::query(SET_PASSWORD_HASH)
            .bind(id.uuid())
            .bind(hash.as_phc());
        execute("set_password_hash", query, &self.pool).await
    }

    async fn mark_email_verified(
        &self,
        id: AccountId,
        at: OffsetDateTime,
    ) -> Result<(), AccountStoreError> {
        let query = sqlx::query(MARK_VERIFIED).bind(id.uuid()).bind(at);
        execute("mark_email_verified", query, &self.pool).await
    }

    async fn set_language(
        &self,
        id: AccountId,
        language: &Language,
    ) -> Result<(), AccountStoreError> {
        let query = sqlx::query(SET_LANGUAGE)
            .bind(id.uuid())
            .bind(language.as_str());
        execute("set_account_language", query, &self.pool).await
    }

    async fn delete(&self, id: AccountId) -> Result<(), AccountStoreError> {
        let query = sqlx::query(DELETE).bind(id.uuid());
        execute("delete_account", query, &self.pool).await
    }
}

/// Runs `query`, timed as `name`.
async fn execute(
    name: &'static str,
    query: Query<'_, Postgres, PgArguments>,
    pool: &PgPool,
) -> Result<(), AccountStoreError> {
    timed(name, query.execute(pool)).await.map_err(database)?;
    Ok(())
}

/// `EmailTaken` when `error` breaks the addresses' unique constraint.
fn taken_or_database(error: sqlx::Error) -> AccountStoreError {
    let is_taken = error.as_database_error().is_some_and(|error| {
        error.is_unique_violation() && error.constraint() == Some(EMAIL_CONSTRAINT)
    });
    if is_taken {
        return AccountStoreError::EmailTaken;
    }
    database(error)
}
