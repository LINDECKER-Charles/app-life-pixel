//! Keyset pagination on `(updated_at, id)` descending: a page reads one row more than it shows,
//! to know whether another page follows.

use life_pixel_service::paging::{Cursor, Page, PageRequest};
use life_pixel_service::ports::StoreError;
use sqlx::{Postgres, QueryBuilder};

/// Adds the cursor's condition, when there is one, then the order and the limit, to a query
/// whose `where` clause is open.
pub(super) fn push_page(builder: &mut QueryBuilder<Postgres>, request: &PageRequest) {
    if let Some(cursor) = request.cursor {
        builder.push(" and (updated_at, id) < (");
        builder.push_bind(cursor.updated_at);
        builder.push(", ");
        builder.push_bind(cursor.id);
        builder.push(")");
    }
    builder.push(" order by updated_at desc, id desc limit ");
    builder.push_bind(i64::from(request.limit) + 1);
}

/// A row a page lists.
pub(super) trait PagedRow: Sized {
    /// The record the row becomes.
    type Record;

    /// Where the row stands in a list.
    fn cursor(&self) -> Cursor;

    /// The row's record.
    fn into_record(self) -> Result<Self::Record, StoreError>;
}

/// The page of `rows`, read with [`push_page`].
pub(super) fn page_of<R: PagedRow>(
    mut rows: Vec<R>,
    request: &PageRequest,
) -> Result<Page<R::Record>, StoreError> {
    let has_more = rows.len() > usize::from(request.limit);
    rows.truncate(usize::from(request.limit));
    let next_cursor = rows.last().filter(|_| has_more).map(PagedRow::cursor);
    let items = rows
        .into_iter()
        .map(PagedRow::into_record)
        .collect::<Result<_, _>>()?;
    Ok(Page { items, next_cursor })
}
