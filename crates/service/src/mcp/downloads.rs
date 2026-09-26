//! The file a signed link names, compiled on demand from the version the link names.

use std::sync::Arc;

use life_pixel_compiler::ExportFile;

use super::McpError;
use super::export_link::{ExportLink, ExportLinks};
use super::ports::AnimationOwners;
use crate::animation::{AnimationEditing, ExportRequest};
use crate::error::CodedError;
use crate::library::{Library, LibraryError};
use crate::owner::Owner;

/// Downloads through signed links: the export is compiled again at each download, and never
/// stored.
#[derive(Clone)]
pub struct ExportDownloads {
    links: ExportLinks,
    owners: Arc<dyn AnimationOwners>,
    library: Library,
    editing: AnimationEditing,
}

impl ExportDownloads {
    /// Downloads checked by `links`, of the animations `owners` knows, read through `library`
    /// and compiled by `editing` — whose events the agent's own export already recorded.
    #[must_use]
    pub fn new(
        links: ExportLinks,
        owners: Arc<dyn AnimationOwners>,
        (library, editing): (Library, AnimationEditing),
    ) -> Self {
        Self {
            links,
            owners,
            library,
            editing,
        }
    }

    /// The file the link `text` names, compiled from the version it names.
    ///
    /// # Errors
    ///
    /// `export.link_invalid` when the link is tampered with or expired, or its animation is gone
    /// or has changed since; the export's own codes; `service.unavailable`.
    pub async fn download(&self, text: &str) -> Result<ExportFile, CodedError> {
        let link = self.links.verify(text)?;
        let owner = self.owners.owner_of(link.animation).await;
        let owner = owner
            .map_err(McpError::from)?
            .ok_or(McpError::LinkInvalid)?;
        let owner = Owner::Account(owner);
        self.check_version(&owner, &link).await?;
        let request = ExportRequest {
            id: link.animation,
            format: link.format,
            tag: link.tag.clone(),
            scale: link.scale,
        };
        let files = self.editing.export(&owner, request).await?;
        self.check_version(&owner, &link).await?;
        let file = files.into_iter().find(|file| file.name == link.file_name);
        Ok(file.ok_or(McpError::LinkInvalid)?)
    }

    /// Whether the animation of `link` is still at the version it names: checked before and
    /// after compiling, versions only grow, so the file is that version's.
    async fn check_version(&self, owner: &Owner, link: &ExportLink) -> Result<(), CodedError> {
        match self.library.get_animation(owner, link.animation).await {
            Ok(record) if record.version == link.version => Ok(()),
            Ok(_) | Err(LibraryError::AnimationNotFound) => Err(McpError::LinkInvalid.into()),
            Err(error) => Err(error.into()),
        }
    }
}
