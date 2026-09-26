//! What an admin does to an account, each action audited: suspending and reactivating it,
//! reading its data export — the right of access —, and erasing it — the right to erasure.

use crate::accounts::DataExport;
use crate::accounts::ports::AccountStatus;
use crate::admin::ports::{Erasure, StatusChange};
use crate::admin::{Admin, AdminError, AdminIdentity, AuditAction, AuditDraft, Reason};
use crate::ids::AccountId;
use crate::owner::Owner;

impl Admin {
    /// Suspends the account `id` for `reason`: its sessions end with it.
    ///
    /// # Errors
    ///
    /// `admin.reason_length`, `admin.user_not_found`, `service.unavailable`.
    pub async fn suspend_user(
        &self,
        admin: &AdminIdentity,
        (id, reason): (AccountId, &str),
    ) -> Result<(), AdminError> {
        let audit = self.draft(admin, (AuditAction::UserSuspend, id.to_string()));
        self.set_status((id, reason, AccountStatus::Suspended), audit)
            .await
    }

    /// Reactivates the account `id` for `reason`.
    ///
    /// # Errors
    ///
    /// `admin.reason_length`, `admin.user_not_found`, `service.unavailable`.
    pub async fn reactivate_user(
        &self,
        admin: &AdminIdentity,
        (id, reason): (AccountId, &str),
    ) -> Result<(), AdminError> {
        let audit = self.draft(admin, (AuditAction::UserReactivate, id.to_string()));
        self.set_status((id, reason, AccountStatus::Active), audit)
            .await
    }

    /// The data export of the account `id`, as its owner would get it (H6); the read is audited
    /// before the export is made.
    ///
    /// # Errors
    ///
    /// `admin.user_not_found`, `service.unavailable`.
    pub async fn export_user(
        &self,
        admin: &AdminIdentity,
        id: AccountId,
    ) -> Result<DataExport, AdminError> {
        self.existing(id).await?;
        let draft = self.draft(admin, (AuditAction::UserExport, id.to_string()));
        self.audit().append(draft.entry(None, None)).await?;
        Ok(self
            .ports
            .accounts
            .export_data(&self.ports.library, id)
            .await?)
    }

    /// Erases the account `id` for `reason`, as its owner's deletion does (H6): its documents
    /// first, then the account and everything it had. The audit entry keeps the account's id and
    /// the reason, never its address.
    ///
    /// # Errors
    ///
    /// `admin.reason_length`, `admin.user_not_found`, `service.unavailable`.
    pub async fn erase_user(
        &self,
        admin: &AdminIdentity,
        (id, reason): (AccountId, &str),
    ) -> Result<(), AdminError> {
        let reason = Reason::parse(reason)?;
        self.existing(id).await?;
        let emptied = self
            .ports
            .library
            .delete_everything(&Owner::Account(id))
            .await;
        emptied.map_err(|_| AdminError::Unavailable)?;
        let mut audit = self.draft(admin, (AuditAction::UserDelete, id.to_string()));
        audit.reason = Some(reason);
        let erased = self.users().erase(Erasure { account: id, audit }).await?;
        erased.then_some(()).ok_or(AdminError::UserNotFound)
    }

    /// Sets the status of the account `id` to `status` for `reason`, audited as `audit` says.
    async fn set_status(
        &self,
        (id, reason, status): (AccountId, &str, AccountStatus),
        mut audit: AuditDraft,
    ) -> Result<(), AdminError> {
        audit.reason = Some(Reason::parse(reason)?);
        let change = StatusChange {
            account: id,
            status,
            audit,
        };
        let changed = self.users().set_status(change).await?;
        changed.then_some(()).ok_or(AdminError::UserNotFound)
    }

    /// Checks that the account `id` exists.
    async fn existing(&self, id: AccountId) -> Result<(), AdminError> {
        let found = self.users().find(id).await?;
        found.map(|_| ()).ok_or(AdminError::UserNotFound)
    }
}
