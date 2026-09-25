//! The storage quota rule, written once for every adapter.

use crate::ports::library_store::StoreError;

/// A change to an owner's storage usage: what a create, a write or a copy asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StorageChange {
    /// The owner's usage before the change, in bytes.
    pub used: u64,
    /// The bytes the change frees: the document a write replaces.
    pub freed: u64,
    /// The bytes the change adds: the new document.
    pub added: u64,
}

impl StorageChange {
    /// The owner's usage after the change.
    #[must_use]
    pub fn usage_after(&self) -> u64 {
        self.used
            .saturating_sub(self.freed)
            .saturating_add(self.added)
    }

    /// Checks the change against `quota`: it is refused when it would take the usage above the
    /// quota *and* above the current usage, so that a write that shrinks a document always passes
    /// and deleting always works. `None`, the local library, means no quota.
    ///
    /// # Errors
    ///
    /// [`StoreError::QuotaExceeded`] with the current usage, the quota, and `requested` — the
    /// bytes the change would add to the usage.
    pub fn check(&self, quota: Option<u64>) -> Result<(), StoreError> {
        let Some(limit) = quota else { return Ok(()) };
        let after = self.usage_after();
        if after <= limit || after <= self.used {
            return Ok(());
        }
        Err(StoreError::QuotaExceeded {
            used: self.used,
            limit,
            requested: after - self.used,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(used: u64, freed: u64, added: u64) -> StorageChange {
        StorageChange { used, freed, added }
    }

    #[test]
    fn no_quota_accepts_everything() {
        assert_eq!(change(u64::MAX - 1, 0, 10).check(None), Ok(()));
    }

    #[test]
    fn a_change_up_to_the_quota_passes() {
        assert_eq!(change(60, 0, 40).check(Some(100)), Ok(()));
        assert_eq!(change(60, 20, 60).check(Some(100)), Ok(()));
    }

    #[test]
    fn a_change_beyond_the_quota_is_refused_with_what_it_adds() {
        let refused = StoreError::QuotaExceeded {
            used: 60,
            limit: 100,
            requested: 41,
        };
        assert_eq!(change(60, 0, 41).check(Some(100)), Err(refused.clone()));
        assert_eq!(change(60, 9, 50).check(Some(100)), Err(refused));
    }

    #[test]
    fn a_change_that_does_not_grow_usage_passes_above_the_quota() {
        assert_eq!(change(150, 30, 20).check(Some(100)), Ok(()));
        assert_eq!(change(150, 30, 30).check(Some(100)), Ok(()));
    }
}
