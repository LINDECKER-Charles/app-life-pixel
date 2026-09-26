//! `PostgresAuditLog`: the `audit_log` table, appended to and read, newest first. A page's cursor
//! carries the last entry's time and its id, in the low bits of a UUID.

use async_trait::async_trait;
use life_pixel_service::admin::ports::{AdminStoreError, AuditLog};
use life_pixel_service::admin::{AuditEntry, AuditFilter, AuditRecord};
use life_pixel_service::paging::{Cursor, Page, PageRequest};
use serde_json::Value;
use sqlx::{PgPool, QueryBuilder};
use time::OffsetDateTime;
use uuid::Uuid;

use super::rows::{PagedRow, corrupt, database, insert_entry, page_of};
use crate::storage::metrics::timed;

const SELECT_ENTRIES: &str = "select id, admin_id, admin_email, action, target_type, target_id, \
                              reason, before::text as before, after::text as after, at \
                              from audit_log where true";

/// The audit log of the migrated database of a pool.
#[derive(Clone)]
pub struct PostgresAuditLog {
    pool: PgPool,
}

impl PostgresAuditLog {
    /// The log of the migrated database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// An entry's row.
#[derive(sqlx::FromRow)]
struct EntryRow {
    id: i64,
    admin_id: String,
    admin_email: String,
    action: String,
    target_type: String,
    target_id: String,
    reason: Option<String>,
    before: Option<String>,
    after: Option<String>,
    at: OffsetDateTime,
}

impl PagedRow for EntryRow {
    type Record = AuditRecord;

    fn cursor(&self) -> Cursor {
        Cursor {
            updated_at: self.at,
            id: Uuid::from_u64_pair(0, self.id.unsigned_abs()),
        }
    }

    fn into_record(self) -> Result<AuditRecord, AdminStoreError> {
        Ok(AuditRecord {
            id: self.id,
            entry: AuditEntry {
                admin_id: self.admin_id,
                admin_email: self.admin_email,
                action: self.action,
                target_type: self.target_type,
                target_id: self.target_id,
                reason: self.reason,
                before: json(self.before.as_deref(), "audit_log.before")?,
                after: json(self.after.as_deref(), "audit_log.after")?,
                at: self.at,
            },
        })
    }
}

/// The JSON of a `jsonb` column read as text.
fn json(text: Option<&str>, column: &str) -> Result<Option<Value>, AdminStoreError> {
    text.map(|text| serde_json::from_str(text).map_err(|_| corrupt(column, &text)))
        .transpose()
}

/// The id of the entry a cursor stopped at.
fn entry_id(cursor: &Cursor) -> i64 {
    let (_, id) = cursor.id.as_u64_pair();
    i64::try_from(id).unwrap_or(i64::MAX)
}

#[async_trait]
impl AuditLog for PostgresAuditLog {
    async fn append(&self, entry: AuditEntry) -> Result<(), AdminStoreError> {
        insert_entry(&self.pool, &entry).await
    }

    async fn list(
        &self,
        filter: AuditFilter,
        page: PageRequest,
    ) -> Result<Page<AuditRecord>, AdminStoreError> {
        let mut builder = QueryBuilder::new(SELECT_ENTRIES);
        let filters = [
            ("admin_id", filter.admin_id),
            ("action", filter.action),
            ("target_id", filter.target_id),
        ];
        for (column, value) in filters {
            if let Some(value) = value {
                builder
                    .push(format_args!(" and {column} = "))
                    .push_bind(value);
            }
        }
        if let Some(cursor) = &page.cursor {
            builder.push(" and id < ").push_bind(entry_id(cursor));
        }
        builder.push(" order by id desc limit ");
        builder.push_bind(i64::from(page.limit) + 1);
        let query = builder.build_query_as::<EntryRow>();
        let rows = timed("admin_select_audit_log", query.fetch_all(&self.pool)).await;
        page_of(rows.map_err(database)?, &page)
    }
}
