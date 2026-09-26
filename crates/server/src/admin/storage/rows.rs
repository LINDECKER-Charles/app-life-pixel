//! What the admin stores share: keyset pages, `like` patterns, the audit entry written in a
//! change's transaction, and their errors.

use life_pixel_service::admin::AuditEntry;
use life_pixel_service::admin::ports::AdminStoreError;
use life_pixel_service::paging::{Cursor, Page, PageRequest};
use serde_json::Value;
use sqlx::{PgExecutor, Postgres, QueryBuilder};

use crate::storage::metrics::timed;

/// The character that escapes `%`, `_` and itself in a `like` pattern.
pub(super) const LIKE_ESCAPE: char = '\\';

const INSERT_ENTRY: &str = "insert into audit_log (admin_id, admin_email, action, target_type, \
                            target_id, reason, before, after, at) \
                            values ($1, $2, $3, $4, $5, $6, $7::jsonb, $8::jsonb, $9)";

/// A row a page lists.
pub(super) trait PagedRow: Sized {
    /// The record the row becomes.
    type Record;

    /// Where the row stands in a list.
    fn cursor(&self) -> Cursor;

    /// The row's record.
    fn into_record(self) -> Result<Self::Record, AdminStoreError>;
}

/// Adds the cursor's condition on `(time, id)`, when there is one, then the order and the limit
/// — one row more than the page shows —, to a query whose `where` clause is open.
pub(super) fn push_page(
    builder: &mut QueryBuilder<Postgres>,
    request: &PageRequest,
    (time, id): (&str, &str),
) {
    if let Some(cursor) = request.cursor {
        builder.push(format_args!(" and ({time}, {id}) < ("));
        builder.push_bind(cursor.updated_at);
        builder.push(", ");
        builder.push_bind(cursor.id);
        builder.push(")");
    }
    builder.push(format_args!(" order by {time} desc, {id} desc limit "));
    builder.push_bind(i64::from(request.limit) + 1);
}

/// The page of `rows`, read with [`push_page`].
pub(super) fn page_of<R: PagedRow>(
    mut rows: Vec<R>,
    request: &PageRequest,
) -> Result<Page<R::Record>, AdminStoreError> {
    let has_more = rows.len() > usize::from(request.limit);
    rows.truncate(usize::from(request.limit));
    let next_cursor = rows.last().filter(|_| has_more).map(PagedRow::cursor);
    let items = rows.into_iter().map(PagedRow::into_record);
    Ok(Page {
        items: items.collect::<Result<_, _>>()?,
        next_cursor,
    })
}

/// The `like` pattern of the texts that hold `text`, its `%`, `_` and escapes escaped.
pub(super) fn contains_pattern(text: &str) -> String {
    let mut pattern = String::with_capacity(text.len() + 2);
    pattern.push('%');
    for character in text.chars() {
        if matches!(character, '%' | '_' | LIKE_ESCAPE) {
            pattern.push(LIKE_ESCAPE);
        }
        pattern.push(character);
    }
    pattern.push('%');
    pattern
}

/// Appends `entry` to the audit log, in the transaction of `executor` when it is one.
pub(super) async fn insert_entry(
    executor: impl PgExecutor<'_>,
    entry: &AuditEntry,
) -> Result<(), AdminStoreError> {
    let query = sqlx::query(INSERT_ENTRY)
        .bind(&entry.admin_id)
        .bind(&entry.admin_email)
        .bind(&entry.action)
        .bind(&entry.target_type)
        .bind(&entry.target_id)
        .bind(entry.reason.as_deref())
        .bind(entry.before.as_ref().map(Value::to_string))
        .bind(entry.after.as_ref().map(Value::to_string))
        .bind(entry.at);
    timed("insert_audit_entry", query.execute(executor))
        .await
        .map_err(database)?;
    Ok(())
}

/// The failure of a query.
pub(super) fn database(error: sqlx::Error) -> AdminStoreError {
    AdminStoreError(format!("database: {error}"))
}

/// A value the schema should have prevented.
pub(super) fn corrupt(column: &str, value: &dyn std::fmt::Display) -> AdminStoreError {
    AdminStoreError(format!("{column} holds an unexpected value: {value}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_search_pattern_escapes_the_wildcards() {
        assert_eq!(contains_pattern("ada"), "%ada%");
        assert_eq!(contains_pattern(r"5%_\"), r"%5\%\_\\%");
    }
}
