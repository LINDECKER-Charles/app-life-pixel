//! An owner's whole library: its storage usage, and its deletion with the account.

use super::{Library, LibraryError};
use crate::owner::Owner;

/// An owner's storage usage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Usage {
    /// The bytes of the owner's documents.
    pub used_bytes: u64,
    /// The quota; `None` for the local library, which has none.
    pub limit_bytes: Option<u64>,
}

impl Library {
    /// The owner's storage usage and quota.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn usage(&self, owner: &Owner) -> Result<Usage, LibraryError> {
        let used = self.store().usage(owner).await;
        let used_bytes = used.map_err(|error| self.refused(owner, error))?;
        Ok(Usage {
            used_bytes,
            limit_bytes: self.quota(owner),
        })
    }

    /// Deletes every project and animation of the owner: for account deletion.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn delete_everything(&self, owner: &Owner) -> Result<(), LibraryError> {
        let deleted = self.store().delete_everything(owner).await;
        deleted.map_err(|error| self.refused(owner, error))
    }
}
