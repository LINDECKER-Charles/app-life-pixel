//! The rows of the accounts' tables, as queries read them, and their conversion into the ports'
//! records.

use life_pixel_service::AccountId;
use life_pixel_service::accounts::ports::{AccountRecord, AccountStatus, AccountStoreError};
use life_pixel_service::accounts::{PasswordHash, TOKEN_HASH_BYTES, TokenHash};
use time::OffsetDateTime;
use uuid::Uuid;

/// The columns of an account's row, in [`AccountRow`]'s order.
macro_rules! account_columns {
    () => {
        "id, email::text as email, password_hash, email_verified_at, language, status, plan, \
         storage_used_bytes, created_at"
    };
}
pub(super) use account_columns;

/// An account's row.
#[derive(sqlx::FromRow)]
pub(super) struct AccountRow {
    id: Uuid,
    email: String,
    password_hash: String,
    email_verified_at: Option<OffsetDateTime>,
    language: String,
    status: String,
    plan: String,
    storage_used_bytes: i64,
    created_at: OffsetDateTime,
}

impl AccountRow {
    /// The record of the row.
    pub(super) fn into_record(self) -> Result<AccountRecord, AccountStoreError> {
        let status = AccountStatus::parse(&self.status)
            .ok_or_else(|| corrupt("accounts.status", &self.status))?;
        let used = u64::try_from(self.storage_used_bytes)
            .map_err(|_| corrupt("accounts.storage_used_bytes", &self.storage_used_bytes))?;
        Ok(AccountRecord {
            id: AccountId::from_uuid(self.id),
            email: self.email,
            password_hash: PasswordHash::from_phc(self.password_hash),
            email_verified_at: self.email_verified_at,
            language: self.language,
            status,
            plan: self.plan,
            storage_used_bytes: used,
            created_at: self.created_at,
        })
    }
}

/// The hash a `bytea` column holds.
pub(super) fn token_hash(bytes: &[u8]) -> Result<TokenHash, AccountStoreError> {
    let bytes = <[u8; TOKEN_HASH_BYTES]>::try_from(bytes)
        .map_err(|_| corrupt("token_hash", &format!("{} bytes", bytes.len())))?;
    Ok(TokenHash::from_bytes(bytes))
}

/// The failure of a query.
pub(super) fn database(error: sqlx::Error) -> AccountStoreError {
    AccountStoreError::Unavailable(format!("database: {error}"))
}

/// A value the schema should have prevented.
fn corrupt(column: &str, value: &dyn std::fmt::Display) -> AccountStoreError {
    AccountStoreError::Unavailable(format!("{column} holds an unexpected value: {value}"))
}
