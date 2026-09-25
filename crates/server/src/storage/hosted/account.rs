//! The account's row within a transaction: every change that moves usage, or deletes, locks it
//! first, so that the changes of one account run one after the other, in the same lock order —
//! account, then project, then animation — and never deadlock.

use life_pixel_service::ports::StoreError;
use life_pixel_service::quota::StorageChange;
use life_pixel_service::{AccountId, Owner};
use sqlx::PgConnection;

use super::rows::{bigint, database, unsigned};
use crate::storage::metrics::timed;

const LOCK_ACCOUNT: &str = "select storage_used_bytes from accounts where id = $1 for update";
const MOVE_USAGE: &str =
    "update accounts set storage_used_bytes = storage_used_bytes + $2 where id = $1";
const RESET_USAGE: &str = "update accounts set storage_used_bytes = 0 where id = $1";

/// The account the hosted store acts for; the local library is not hosted.
pub(super) fn account_of(owner: &Owner) -> Result<AccountId, StoreError> {
    owner.account().ok_or_else(|| {
        StoreError::Unavailable("the hosted library stores accounts only".to_owned())
    })
}

/// Locks the account's row until the transaction ends, and returns its usage; `None` when the
/// account does not exist, and so owns nothing.
pub(super) async fn lock_usage(
    connection: &mut PgConnection,
    account: AccountId,
) -> Result<Option<u64>, StoreError> {
    let query = sqlx::query_scalar::<_, i64>(LOCK_ACCOUNT).bind(account.uuid());
    let used = timed("lock_account", query.fetch_optional(connection)).await;
    used.map_err(database)?
        .map(|used| unsigned(used, "accounts.storage_used_bytes"))
        .transpose()
}

/// The bytes `change` adds to the usage, negative when it frees more than it adds.
pub(super) fn usage_delta(change: &StorageChange) -> Result<i64, StoreError> {
    Ok(bigint(change.added)? - bigint(change.freed)?)
}

/// Adds `delta` bytes, negative when freed, to the account's usage.
pub(super) async fn add_usage(
    connection: &mut PgConnection,
    account: AccountId,
    delta: i64,
) -> Result<(), StoreError> {
    if delta == 0 {
        return Ok(());
    }
    let query = sqlx::query(MOVE_USAGE).bind(account.uuid()).bind(delta);
    timed("move_usage", query.execute(connection))
        .await
        .map_err(database)?;
    Ok(())
}

/// Sets the account's usage to zero: it holds no document any more.
pub(super) async fn reset_usage(
    connection: &mut PgConnection,
    account: AccountId,
) -> Result<(), StoreError> {
    let query = sqlx::query(RESET_USAGE).bind(account.uuid());
    timed("reset_usage", query.execute(connection))
        .await
        .map_err(database)?;
    Ok(())
}
