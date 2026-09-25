//! What the library commands answer: H6's Project, Animation and Page, so that the app reads
//! the desktop's answers as it reads the API's.

use life_pixel_service::Page;
use life_pixel_service::library::Usage;
use life_pixel_service::ports::{AnimationRecord, ProjectRecord};
use serde::Serialize;
use time::OffsetDateTime;

/// A project, as H6 shapes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    id: String,
    name: String,
    animation_count: u32,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
}

impl From<ProjectRecord> for Project {
    fn from(record: ProjectRecord) -> Self {
        Self {
            id: record.id.to_string(),
            name: record.name.as_str().to_owned(),
            animation_count: record.animation_count,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// An animation without its document, as H6 shapes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Animation {
    /// Its id.
    pub id: String,
    /// The project it belongs to.
    pub project_id: String,
    title: String,
    width: u16,
    height: u16,
    frame_count: u16,
    document_bytes: u64,
    /// Its document's version: what a save must name.
    pub version: u64,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    updated_at: OffsetDateTime,
}

impl From<AnimationRecord> for Animation {
    fn from(record: AnimationRecord) -> Self {
        Self {
            id: record.id.to_string(),
            project_id: record.project.to_string(),
            title: record.meta.title.as_str().to_owned(),
            width: record.meta.width,
            height: record.meta.height,
            frame_count: record.meta.frame_count,
            document_bytes: record.document_bytes,
            version: record.version,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

/// A page of a list: its items, and the cursor of the next page, `null` on the last.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPage<T> {
    /// The items, from the most recently updated.
    pub items: Vec<T>,
    /// Where the next page starts.
    pub next_cursor: Option<String>,
}

impl<T> ListPage<T> {
    /// The page of `page`'s records, each shaped as `T`.
    #[must_use]
    pub fn of<R: Into<T>>(page: Page<R>) -> Self {
        Self {
            items: page.items.into_iter().map(Into::into).collect(),
            next_cursor: page.next_cursor.map(|cursor| cursor.to_string()),
        }
    }
}

/// An opened animation: its summary and its document, in base64.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OpenedDocument {
    /// The animation.
    pub summary: Animation,
    /// Its document, in base64.
    pub document: String,
}

/// The library's storage usage; the local library has no quota.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageUsage {
    /// The bytes of the documents.
    pub used_bytes: u64,
    /// `null`: no quota.
    pub limit_bytes: Option<u64>,
}

impl From<Usage> for StorageUsage {
    fn from(usage: Usage) -> Self {
        Self {
            used_bytes: usage.used_bytes,
            limit_bytes: usage.limit_bytes,
        }
    }
}
