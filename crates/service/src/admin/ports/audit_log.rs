//! The audit log: appended to, read, never changed.

use async_trait::async_trait;

use super::AdminStoreError;
use crate::admin::{AuditEntry, AuditFilter, AuditRecord};
use crate::paging::{Page, PageRequest};

/// The audit log. The changes write their own entries, in their transactions; this port appends
/// the entries of reads, such as a data export, and lists them all.
#[async_trait]
pub trait AuditLog: Send + Sync {
    /// Appends `entry`.
    async fn append(&self, entry: AuditEntry) -> Result<(), AdminStoreError>;

    /// A page of the entries `filter` keeps, newest first.
    async fn list(
        &self,
        filter: AuditFilter,
        page: PageRequest,
    ) -> Result<Page<AuditRecord>, AdminStoreError>;
}
