//! The library's bodies: projects, animations and their pages as the routes answer them, and
//! what the routes read.

use life_pixel_service::paging::{Cursor, Page as ServicePage, PageRequest};
use life_pixel_service::ports::library_store::{AnimationRecord, ProjectRecord};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::http::problem::Problem;

/// A project of the account's library.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Its id.
    #[schema(format = "uuid")]
    pub id: String,
    /// Its name.
    pub name: String,
    /// How many animations it holds.
    pub animation_count: u32,
    /// When it was created.
    #[schema(format = DateTime)]
    pub created_at: String,
    /// When it or one of its animations last changed.
    #[schema(format = DateTime)]
    pub updated_at: String,
}

impl From<ProjectRecord> for Project {
    fn from(record: ProjectRecord) -> Self {
        Self {
            id: record.id.uuid().to_string(),
            name: record.name.as_str().to_owned(),
            animation_count: record.animation_count,
            created_at: timestamp(record.created_at),
            updated_at: timestamp(record.updated_at),
        }
    }
}

/// An animation of the account's library, without its document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Animation {
    /// Its id.
    #[schema(format = "uuid")]
    pub id: String,
    /// The project holding it.
    #[schema(format = "uuid")]
    pub project_id: String,
    /// Its title.
    pub title: String,
    /// The canvas width, in pixels.
    pub width: u16,
    /// The canvas height, in pixels.
    pub height: u16,
    /// Its frames.
    pub frame_count: u16,
    /// The size of its document, in bytes: what it counts against the quota.
    pub document_bytes: u64,
    /// Its version: the `ETag` of its document, which a write names in `If-Match`.
    pub version: u64,
    /// When it was created.
    #[schema(format = DateTime)]
    pub created_at: String,
    /// When its document last changed, or it moved.
    #[schema(format = DateTime)]
    pub updated_at: String,
}

impl From<AnimationRecord> for Animation {
    fn from(record: AnimationRecord) -> Self {
        Self {
            id: record.id.uuid().to_string(),
            project_id: record.project.uuid().to_string(),
            title: record.meta.title.as_str().to_owned(),
            width: record.meta.width,
            height: record.meta.height,
            frame_count: record.meta.frame_count,
            document_bytes: record.document_bytes,
            version: record.version,
            created_at: timestamp(record.created_at),
            updated_at: timestamp(record.updated_at),
        }
    }
}

/// A page of a list, from the most recently updated item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Page<T: ToSchema> {
    /// The items.
    pub items: Vec<T>,
    /// The `cursor` of the next page; `null` on the last one.
    pub next_cursor: Option<String>,
}

impl<T: ToSchema> Page<T> {
    /// The page of `page`, each record turned into its item.
    pub fn of<R>(page: ServicePage<R>, item: impl Fn(R) -> T) -> Self {
        Self {
            items: page.items.into_iter().map(item).collect(),
            next_cursor: page.next_cursor.map(|cursor| cursor.to_string()),
        }
    }
}

/// Which page of the projects to read.
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ProjectQuery {
    /// The `nextCursor` of the previous page; none for the first.
    pub cursor: Option<String>,
    /// The most items of the page: 1 to 100, 50 by default.
    pub limit: Option<u16>,
}

impl ProjectQuery {
    /// The page asked for.
    ///
    /// # Errors
    ///
    /// `request.malformed` for a cursor that does not decode.
    pub fn page(&self) -> Result<PageRequest, Problem> {
        page_request(self.cursor.as_deref(), self.limit)
    }
}

/// Which animations to list, and which page.
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AnimationQuery {
    /// Only the animations of this project.
    pub project: Option<Uuid>,
    /// Only the animations whose title holds this text, whatever its case.
    pub q: Option<String>,
    /// The `nextCursor` of the previous page; none for the first.
    pub cursor: Option<String>,
    /// The most items of the page: 1 to 100, 50 by default.
    pub limit: Option<u16>,
}

impl AnimationQuery {
    /// The page asked for.
    ///
    /// # Errors
    ///
    /// `request.malformed` for a cursor that does not decode.
    pub fn page(&self) -> Result<PageRequest, Problem> {
        page_request(self.cursor.as_deref(), self.limit)
    }
}

/// A project's name: to create, rename or duplicate it.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProjectName {
    /// The name.
    pub name: String,
}

/// A change of an animation: its title — a save, under `If-Match` —, its project, or both.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnimationChange {
    /// The new title.
    pub title: Option<String>,
    /// The project to move it to.
    pub project_id: Option<Uuid>,
}

/// A copy of an animation.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnimationCopy {
    /// The copy's title.
    pub title: String,
    /// The project of the copy; the original's when absent.
    pub project_id: Option<Uuid>,
}

/// A document, as the model serializes it: the format of the local library's files.
#[derive(ToSchema)]
#[schema(value_type = Object)]
pub struct AnimationDocument(pub serde_json::Value);

/// `at` in RFC 3339.
fn timestamp(at: OffsetDateTime) -> String {
    at.format(&Rfc3339).unwrap_or_default()
}

/// The page after `cursor`, of `limit` items.
fn page_request(cursor: Option<&str>, limit: Option<u16>) -> Result<PageRequest, Problem> {
    let cursor = cursor.map(str::parse::<Cursor>).transpose()?;
    Ok(PageRequest::new(cursor, limit))
}
