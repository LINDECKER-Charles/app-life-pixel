//! The rows of the index, as queries read them, and their conversion into the port's records.

use life_pixel_core::Name;
use life_pixel_service::paging::Cursor;
use life_pixel_service::ports::StoreError;
use life_pixel_service::ports::library_store::{AnimationMeta, AnimationRecord, ProjectRecord};
use life_pixel_service::{AnimationId, ProjectId};
use time::OffsetDateTime;
use uuid::Uuid;

use super::paging::PagedRow;

/// The columns of an animation's row, in [`AnimationRow`]'s order.
macro_rules! animation_columns {
    () => {
        "id, project_id, title, width, height, frame_count, document_key, document_bytes, \
         version, created_at, updated_at"
    };
}
pub(super) use animation_columns;

/// The columns of a project's row with its animations counted, `p` being `projects`.
macro_rules! project_columns {
    () => {
        "p.id, p.name, p.created_at, p.updated_at, \
         (select count(*) from animations a where a.project_id = p.id) as animation_count"
    };
}
pub(super) use project_columns;

/// A project's row.
#[derive(sqlx::FromRow)]
pub(super) struct ProjectRow {
    id: Uuid,
    name: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    animation_count: i64,
}

impl PagedRow for ProjectRow {
    type Record = ProjectRecord;

    fn cursor(&self) -> Cursor {
        Cursor {
            updated_at: self.updated_at,
            id: self.id,
        }
    }

    fn into_record(self) -> Result<ProjectRecord, StoreError> {
        Ok(ProjectRecord {
            id: ProjectId::from_uuid(self.id),
            name: Name::new(&self.name).map_err(|_| corrupt("projects.name"))?,
            animation_count: u32::try_from(self.animation_count).unwrap_or(u32::MAX),
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// An animation's row.
#[derive(sqlx::FromRow)]
pub(super) struct AnimationRow {
    id: Uuid,
    project_id: Uuid,
    title: String,
    width: i32,
    height: i32,
    frame_count: i32,
    /// Where its document is stored.
    pub(super) document_key: String,
    document_bytes: i64,
    version: i64,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl PagedRow for AnimationRow {
    type Record = AnimationRecord;

    fn cursor(&self) -> Cursor {
        Cursor {
            updated_at: self.updated_at,
            id: self.id,
        }
    }

    fn into_record(self) -> Result<AnimationRecord, StoreError> {
        let meta = AnimationMeta {
            title: Name::new(&self.title).map_err(|_| corrupt("animations.title"))?,
            width: small(self.width, "animations.width")?,
            height: small(self.height, "animations.height")?,
            frame_count: small(self.frame_count, "animations.frame_count")?,
        };
        Ok(AnimationRecord {
            id: AnimationId::from_uuid(self.id),
            project: ProjectId::from_uuid(self.project_id),
            meta,
            document_bytes: unsigned(self.document_bytes, "animations.document_bytes")?,
            version: unsigned(self.version, "animations.version")?,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// The version of a new document; each write adds one.
pub(super) const FIRST_VERSION: i64 = 1;

/// `value` as a column of type `bigint`.
pub(super) fn bigint(value: u64) -> Result<i64, StoreError> {
    i64::try_from(value).map_err(|_| StoreError::Unavailable(format!("{value} exceeds a bigint")))
}

/// A `bigint` column that cannot be negative.
pub(super) fn unsigned(value: i64, column: &str) -> Result<u64, StoreError> {
    u64::try_from(value).map_err(|_| corrupt(column))
}

/// An `integer` column holding a canvas side or a frame count.
fn small(value: i32, column: &str) -> Result<u16, StoreError> {
    u16::try_from(value).map_err(|_| corrupt(column))
}

/// A row that breaks the rules the store writes by.
fn corrupt(column: &str) -> StoreError {
    StoreError::Unavailable(format!("the index holds an invalid {column}"))
}

/// A database failure, its detail kept for the log.
pub(super) fn database(error: sqlx::Error) -> StoreError {
    StoreError::Unavailable(format!("database: {error}"))
}

/// An object storage failure, its detail kept for the log.
pub(super) fn objects(error: object_store::Error) -> StoreError {
    StoreError::Unavailable(format!("object storage: {error}"))
}
