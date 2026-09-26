//! What the admin changes on an account, each change and its audit entry in one transaction:
//! the status, the sessions a suspension ends, and the erasure.

use life_pixel_service::AccountId;
use life_pixel_service::accounts::ports::AccountStatus;
use life_pixel_service::admin::ports::{
    AdminStoreError, Erasure, StatusChange, erased_state, status_state,
};
use sqlx::PgExecutor;

use super::rows::{corrupt, database, insert_entry};
use super::users::{PostgresAdminUserStore, select_user};
use crate::storage::metrics::timed;

const LOCK_STATUS: &str = "select status from accounts where id = $1 for update";
const UPDATE_STATUS: &str = "update accounts set status = $2 where id = $1";
const END_SESSIONS: &str = "delete from sessions where account_id = $1";
const DELETE_ACCOUNT: &str = "delete from accounts where id = $1";

impl PostgresAdminUserStore {
    /// Sets the status of `change`, ends the sessions of a suspended account, and writes the
    /// audit entry, in one transaction.
    pub(super) async fn change_status(
        &self,
        change: StatusChange,
    ) -> Result<bool, AdminStoreError> {
        let mut transaction = self.pool.begin().await.map_err(database)?;
        let Some(before) = lock_status(&mut *transaction, change.account).await? else {
            return Ok(false);
        };
        let query = sqlx::query(UPDATE_STATUS)
            .bind(change.account.uuid())
            .bind(change.status.as_str());
        timed("admin_update_status", query.execute(&mut *transaction))
            .await
            .map_err(database)?;
        if change.status == AccountStatus::Suspended {
            let query = sqlx::query(END_SESSIONS).bind(change.account.uuid());
            timed("admin_end_sessions", query.execute(&mut *transaction))
                .await
                .map_err(database)?;
        }
        let entry = change.audit.entry(
            Some(status_state(before)),
            Some(status_state(change.status)),
        );
        insert_entry(&mut *transaction, &entry).await?;
        transaction.commit().await.map_err(database)?;
        Ok(true)
    }

    /// Deletes the account of `erasure` — its sessions, tokens, rows and support requests go with
    /// it — and writes the audit entry of what it was, never its address, in one transaction.
    pub(super) async fn delete_account(&self, erasure: Erasure) -> Result<bool, AdminStoreError> {
        let mut transaction = self.pool.begin().await.map_err(database)?;
        if lock_status(&mut *transaction, erasure.account)
            .await?
            .is_none()
        {
            return Ok(false);
        }
        let Some(user) = select_user(&mut *transaction, erasure.account).await? else {
            return Ok(false);
        };
        let query = sqlx::query(DELETE_ACCOUNT).bind(erasure.account.uuid());
        timed("admin_delete_account", query.execute(&mut *transaction))
            .await
            .map_err(database)?;
        let entry = erasure.audit.entry(Some(erased_state(&user)), None);
        insert_entry(&mut *transaction, &entry).await?;
        transaction.commit().await.map_err(database)?;
        Ok(true)
    }
}

/// The status of the account `id`, locked until the transaction ends; `None` when it does not
/// exist.
async fn lock_status(
    executor: impl PgExecutor<'_>,
    id: AccountId,
) -> Result<Option<AccountStatus>, AdminStoreError> {
    let query = sqlx::query_scalar::<_, String>(LOCK_STATUS).bind(id.uuid());
    let status = timed("admin_lock_account", query.fetch_optional(executor)).await;
    let Some(status) = status.map_err(database)? else {
        return Ok(None);
    };
    let parsed = AccountStatus::parse(&status);
    parsed
        .map(Some)
        .ok_or_else(|| corrupt("accounts.status", &status))
}
