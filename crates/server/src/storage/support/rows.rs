//! The rows of the support tables, as queries read them, and their conversion into the ports'
//! records.

use life_pixel_service::paging::Cursor;
use life_pixel_service::support::ports::{SupportMessage, SupportRequest, SupportStoreError};
use life_pixel_service::support::{Author, Category, SupportRequestId, SupportStatus};
use time::OffsetDateTime;
use uuid::Uuid;

/// The columns of a request's row, in [`RequestRow`]'s order.
macro_rules! request_columns {
    () => {
        "id, category, status, screenshot_key is not null as has_screenshot, created_at, \
         updated_at"
    };
}
pub(super) use request_columns;

/// A request's row, as its author sees it.
#[derive(sqlx::FromRow)]
pub(super) struct RequestRow {
    id: Uuid,
    category: String,
    status: String,
    has_screenshot: bool,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl RequestRow {
    /// Where the row stands in a list.
    pub(super) fn cursor(&self) -> Cursor {
        Cursor {
            updated_at: self.updated_at,
            id: self.id,
        }
    }

    /// The record of the row.
    pub(super) fn into_record(self) -> Result<SupportRequest, SupportStoreError> {
        let category = Category::parse(&self.category)
            .map_err(|_| corrupt("support_requests.category", &self.category))?;
        let status = SupportStatus::parse(&self.status)
            .ok_or_else(|| corrupt("support_requests.status", &self.status))?;
        Ok(SupportRequest {
            id: SupportRequestId::from_uuid(self.id),
            category,
            status,
            has_screenshot: self.has_screenshot,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

/// A message's row.
#[derive(sqlx::FromRow)]
pub(super) struct MessageRow {
    id: Uuid,
    author: String,
    body: String,
    created_at: OffsetDateTime,
}

impl MessageRow {
    /// The record of the row.
    pub(super) fn into_record(self) -> Result<SupportMessage, SupportStoreError> {
        let author = Author::parse(&self.author)
            .ok_or_else(|| corrupt("support_messages.author", &self.author))?;
        Ok(SupportMessage {
            id: self.id,
            author,
            body: self.body,
            created_at: self.created_at,
        })
    }
}

/// The failure of a query.
pub(super) fn database(error: sqlx::Error) -> SupportStoreError {
    SupportStoreError(format!("database: {error}"))
}

/// A value the schema should have prevented.
fn corrupt(column: &str, value: &str) -> SupportStoreError {
    SupportStoreError(format!("{column} holds an unexpected value: {value}"))
}
