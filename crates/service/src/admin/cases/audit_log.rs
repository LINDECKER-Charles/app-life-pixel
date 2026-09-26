//! Reading the audit log.

use crate::admin::{Admin, AdminError, AuditFilter, AuditRecord};
use crate::paging::{Page, PageRequest};

impl Admin {
    /// A page of the entries `filter` keeps, newest first.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn audit_log(
        &self,
        filter: AuditFilter,
        page: PageRequest,
    ) -> Result<Page<AuditRecord>, AdminError> {
        Ok(self.audit().list(filter, page).await?)
    }
}
