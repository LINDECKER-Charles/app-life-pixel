//! Why a library use case failed, as a code the interface translates.

use life_pixel_core::DocumentError;
use serde_json::{Map, Value};
use thiserror::Error;

use crate::error::{Coded, CodedError, params};
use crate::ports::library_store::StoreError;

/// The error of a [`Library`](super::Library) use case.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum LibraryError {
    /// The document, a name or a title breaks a rule of `core`: its code.
    #[error(transparent)]
    Document(#[from] DocumentError),
    /// `library.project_not_found`.
    #[error("project not found")]
    ProjectNotFound,
    /// `library.animation_not_found`.
    #[error("animation not found")]
    AnimationNotFound,
    /// `document.version_conflict`: the document changed since the caller read it.
    #[error("version conflict, current version {current}")]
    VersionConflict {
        /// The document's current version.
        current: u64,
    },
    /// `quota.storage_exceeded`.
    #[error("storage quota exceeded: {used} + {requested} > {limit} bytes")]
    QuotaExceeded {
        /// The owner's usage, in bytes.
        used: u64,
        /// The quota, in bytes.
        limit: u64,
        /// The bytes the change would add to the usage.
        requested: u64,
    },
    /// `service.unavailable`: the storage or a worker failed; the detail is logged.
    #[error("service unavailable")]
    Unavailable,
}

impl From<StoreError> for LibraryError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::ProjectNotFound => Self::ProjectNotFound,
            StoreError::AnimationNotFound => Self::AnimationNotFound,
            StoreError::VersionConflict { current } => Self::VersionConflict { current },
            StoreError::QuotaExceeded {
                used,
                limit,
                requested,
            } => Self::QuotaExceeded {
                used,
                limit,
                requested,
            },
            StoreError::Unavailable(detail) => {
                tracing::error!(%detail, "library storage unavailable");
                Self::Unavailable
            }
        }
    }
}

impl Coded for LibraryError {
    fn code(&self) -> &'static str {
        match self {
            Self::Document(error) => error.code(),
            Self::ProjectNotFound => "library.project_not_found",
            Self::AnimationNotFound => "library.animation_not_found",
            Self::VersionConflict { .. } => "document.version_conflict",
            Self::QuotaExceeded { .. } => "quota.storage_exceeded",
            Self::Unavailable => "service.unavailable",
        }
    }

    fn params(&self) -> Map<String, Value> {
        match self {
            Self::Document(error) => error.params(),
            Self::VersionConflict { current } => params([("current", (*current).into())]),
            Self::QuotaExceeded {
                used,
                limit,
                requested,
            } => params([
                ("used", (*used).into()),
                ("limit", (*limit).into()),
                ("requested", (*requested).into()),
            ]),
            Self::ProjectNotFound | Self::AnimationNotFound | Self::Unavailable => Map::new(),
        }
    }
}

impl From<LibraryError> for CodedError {
    fn from(error: LibraryError) -> Self {
        Self::of(&error)
    }
}
