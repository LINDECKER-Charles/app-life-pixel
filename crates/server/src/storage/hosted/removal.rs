//! Deleting an animation or everything an account holds, and reading its usage. Deleting always
//! passes the quota: it only frees bytes.

use life_pixel_service::ports::library_store::StoreError;
use life_pixel_service::{AccountId, AnimationId};
use object_store::path::Path;

use super::HostedLibraryStore;
use super::account::{add_usage, lock_usage, reset_usage};
use super::rows::{database, unsigned};
use crate::storage::metrics::timed;

const DELETE_ANIMATION: &str = "delete from animations where id = $1 and account_id = $2 \
                                returning document_key, document_bytes";
const DELETE_ALL_ANIMATIONS: &str =
    "delete from animations where account_id = $1 returning document_key";
const DELETE_ALL_PROJECTS: &str = "delete from projects where account_id = $1";
const SELECT_USAGE: &str = "select storage_used_bytes from accounts where id = $1";

impl HostedLibraryStore {
    /// Deletes the animation's row and frees its bytes, then deletes its document.
    pub(super) async fn remove_animation(
        &self,
        account: AccountId,
        id: AnimationId,
    ) -> Result<(), StoreError> {
        let mut transaction = self.begin().await?;
        let used = lock_usage(&mut transaction, account).await?;
        used.ok_or(StoreError::AnimationNotFound)?;
        let query = sqlx::query_as::<_, (String, i64)>(DELETE_ANIMATION)
            .bind(id.uuid())
            .bind(account.uuid());
        let deleted = timed("delete_animation", query.fetch_optional(&mut *transaction)).await;
        let (key, bytes) = deleted
            .map_err(database)?
            .ok_or(StoreError::AnimationNotFound)?;
        add_usage(&mut transaction, account, -bytes).await?;
        transaction.commit().await.map_err(database)?;
        self.delete_objects([Path::from(key)]).await;
        Ok(())
    }

    /// Deletes every project and animation of the account, sets its usage to zero, then deletes
    /// the documents. The account's row stays: deleting it is the account's business.
    pub(super) async fn remove_everything(&self, account: AccountId) -> Result<(), StoreError> {
        let mut transaction = self.begin().await?;
        if lock_usage(&mut transaction, account).await?.is_none() {
            return Ok(());
        }
        let animations =
            sqlx::query_scalar::<_, String>(DELETE_ALL_ANIMATIONS).bind(account.uuid());
        let keys = timed(
            "delete_all_animations",
            animations.fetch_all(&mut *transaction),
        )
        .await
        .map_err(database)?;
        let projects = sqlx::query(DELETE_ALL_PROJECTS).bind(account.uuid());
        timed("delete_all_projects", projects.execute(&mut *transaction))
            .await
            .map_err(database)?;
        reset_usage(&mut transaction, account).await?;
        transaction.commit().await.map_err(database)?;
        self.delete_objects(keys.into_iter().map(Path::from)).await;
        Ok(())
    }

    /// The bytes of the account's documents; none for an account that does not exist.
    pub(super) async fn select_usage(&self, account: AccountId) -> Result<u64, StoreError> {
        let query = sqlx::query_scalar::<_, i64>(SELECT_USAGE).bind(account.uuid());
        let used = timed("select_usage", query.fetch_optional(&self.pool)).await;
        let used = used.map_err(database)?.unwrap_or_default();
        unsigned(used, "accounts.storage_used_bytes")
    }
}
