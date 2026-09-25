//! Documents: open, save with a version check, rename — a title lives in its document.

use bytes::Bytes;
use life_pixel_core::Name;

use super::parsing::{self, Document};
use super::{Library, LibraryError};
use crate::ids::AnimationId;
use crate::owner::Owner;
use crate::ports::library_store::{AnimationRecord, DocumentWrite};

impl Library {
    /// The animation `id` and its document.
    ///
    /// # Errors
    ///
    /// `library.animation_not_found`; `service.unavailable`.
    pub async fn open_document(
        &self,
        owner: &Owner,
        id: AnimationId,
    ) -> Result<(AnimationRecord, Bytes), LibraryError> {
        let opened = self.store().read_document(owner, id).await;
        opened.map_err(|error| self.refused(owner, error))
    }

    /// Replaces the document of `id` when its version is still `expected_version`. The size is
    /// checked against `MAX_DOCUMENT_BYTES`, then the document is parsed and validated by `core`
    /// before anything is written.
    ///
    /// # Errors
    ///
    /// The `document.*` code of an invalid document; `library.animation_not_found`;
    /// `document.version_conflict`; `quota.storage_exceeded`; `service.unavailable`.
    #[allow(clippy::too_many_arguments)] // The use case's signature is the contract of service.md.
    pub async fn save_document(
        &self,
        owner: &Owner,
        id: AnimationId,
        expected_version: u64,
        document: Bytes,
    ) -> Result<AnimationRecord, LibraryError> {
        let document = parsing::parse(document).await?;
        let write = self.document_write((id, expected_version), document);
        self.write(owner, write).await
    }

    /// Retitles the animation `id` when its version is still `expected_version`: a save.
    ///
    /// # Errors
    ///
    /// `document.name` for an invalid title; `library.animation_not_found`;
    /// `document.version_conflict`; `quota.storage_exceeded`; `service.unavailable`.
    #[allow(clippy::too_many_arguments)] // The use case's signature is the contract of service.md.
    pub async fn rename_animation(
        &self,
        owner: &Owner,
        id: AnimationId,
        expected_version: u64,
        title: &str,
    ) -> Result<AnimationRecord, LibraryError> {
        let title = Name::new(title)?;
        let (current, bytes) = self.open_document(owner, id).await?;
        if current.version != expected_version {
            return Err(LibraryError::VersionConflict {
                current: current.version,
            });
        }
        let document = parsing::retitle(bytes, title).await?;
        let write = self.document_write((id, expected_version), document);
        self.write(owner, write).await
    }

    /// The write of `document` over the animation's `(id, expected_version)`, now.
    fn document_write(
        &self,
        (id, expected_version): (AnimationId, u64),
        document: Document,
    ) -> DocumentWrite {
        DocumentWrite {
            id,
            expected_version,
            meta: document.meta,
            document: document.bytes,
            at: self.ports.clock.now(),
        }
    }

    /// Stores `write` under the owner's quota.
    async fn write(
        &self,
        owner: &Owner,
        write: DocumentWrite,
    ) -> Result<AnimationRecord, LibraryError> {
        let quota = self.quota(owner);
        let saved = self.store().write_document(owner, write, quota).await;
        let saved = saved.map_err(|error| self.refused(owner, error))?;
        self.record_saved(owner, saved.document_bytes);
        Ok(saved)
    }
}
