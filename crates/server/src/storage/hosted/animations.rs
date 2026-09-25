//! Animations: create with their first document, read, list and search, move.

use life_pixel_service::paging::{Page, PageRequest};
use life_pixel_service::ports::library_store::{
    AnimationFilter, AnimationRecord, NewAnimationRecord, StoreError,
};
use life_pixel_service::quota::StorageChange;
use life_pixel_service::{AccountId, AnimationId, ProjectId};
use object_store::path::Path;
use sqlx::{PgConnection, QueryBuilder};
use time::OffsetDateTime;

use super::account::{add_usage, lock_usage, usage_delta};
use super::paging::{PagedRow, page_of, push_page};
use super::projects::require_project;
use super::rows::{AnimationRow, FIRST_VERSION, animation_columns, bigint, database};
use super::{HostedLibraryStore, StagedDocument};
use crate::storage::keys::new_document_key;
use crate::storage::metrics::timed;

const INSERT_ANIMATION: &str = concat!(
    "insert into animations (id, account_id, project_id, title, width, height, frame_count, \
     document_key, document_bytes, version, created_at, updated_at) \
     values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $11) returning ",
    animation_columns!()
);
const SELECT_ANIMATION: &str = concat!(
    "select ",
    animation_columns!(),
    " from animations where id = $1 and account_id = $2"
);
const SELECT_ANIMATIONS: &str = concat!(
    "select ",
    animation_columns!(),
    " from animations where account_id = "
);
const LOCK_ANIMATION: &str = concat!(
    "select ",
    animation_columns!(),
    " from animations where id = $1 and account_id = $2 for update"
);
const MOVE_ANIMATION: &str = concat!(
    "update animations set project_id = $2, updated_at = $3 where id = $1 returning ",
    animation_columns!()
);
/// The character that escapes `%`, `_` and itself in a `like` pattern.
const LIKE_ESCAPE: char = '\\';

impl HostedLibraryStore {
    /// Stores the first document, then records the animation; the document is deleted when the
    /// record fails.
    pub(super) async fn insert_animation(
        &self,
        account: AccountId,
        (new, quota): (NewAnimationRecord, Option<u64>),
    ) -> Result<AnimationRecord, StoreError> {
        let key = new_document_key(account, new.id);
        self.put_object(&key, new.document.clone()).await?;
        let staged = StagedDocument {
            account,
            key: &key,
            quota,
        };
        let recorded = self.record_animation(&staged, &new).await;
        if recorded.is_err() {
            self.delete_objects([key]).await;
        }
        recorded
    }

    /// The transaction of a creation: locks the account and the project, checks the quota,
    /// inserts the row and moves the usage.
    async fn record_animation(
        &self,
        staged: &StagedDocument<'_>,
        new: &NewAnimationRecord,
    ) -> Result<AnimationRecord, StoreError> {
        let StagedDocument {
            account,
            key,
            quota,
        } = *staged;
        let mut transaction = self.begin().await?;
        let used = lock_usage(&mut transaction, account).await?;
        let used = used.ok_or(StoreError::ProjectNotFound)?;
        require_project(&mut transaction, account, new.project).await?;
        let added = u64::try_from(new.document.len()).unwrap_or(u64::MAX);
        let change = StorageChange {
            used,
            freed: 0,
            added,
        };
        change.check(quota)?;
        let row = insert_row(&mut transaction, (account, new), key).await?;
        add_usage(&mut transaction, account, usage_delta(&change)?).await?;
        transaction.commit().await.map_err(database)?;
        row.into_record()
    }

    /// The animation's row, whether or not it is locked by a write.
    pub(super) async fn select_animation(
        &self,
        account: AccountId,
        id: AnimationId,
    ) -> Result<AnimationRow, StoreError> {
        let query = sqlx::query_as::<_, AnimationRow>(SELECT_ANIMATION)
            .bind(id.uuid())
            .bind(account.uuid());
        let row = timed("select_animation", query.fetch_optional(&self.pool)).await;
        row.map_err(database)?.ok_or(StoreError::AnimationNotFound)
    }

    /// A page of the account's animations, of one project or all, whose title holds the query:
    /// an `ilike` the trigram index serves.
    pub(super) async fn select_animations(
        &self,
        account: AccountId,
        (filter, request): (&AnimationFilter, &PageRequest),
    ) -> Result<Page<AnimationRecord>, StoreError> {
        let mut builder = QueryBuilder::new(SELECT_ANIMATIONS);
        builder.push_bind(account.uuid());
        if let Some(project) = filter.project {
            builder.push(" and project_id = ").push_bind(project.uuid());
        }
        if let Some(query) = &filter.query {
            builder
                .push(" and title ilike ")
                .push_bind(contains_pattern(query));
            builder.push(format_args!(" escape '{LIKE_ESCAPE}'"));
        }
        push_page(&mut builder, request);
        let query = builder.build_query_as::<AnimationRow>();
        let rows = timed("list_animations", query.fetch_all(&self.pool)).await;
        let rows = rows.map_err(database)?;
        page_of(rows, request)
    }

    /// Moves the animation to the project `to`, both locked.
    pub(super) async fn update_animation_project(
        &self,
        (account, id): (AccountId, AnimationId),
        (to, at): (ProjectId, OffsetDateTime),
    ) -> Result<AnimationRecord, StoreError> {
        let mut transaction = self.begin().await?;
        let used = lock_usage(&mut transaction, account).await?;
        used.ok_or(StoreError::AnimationNotFound)?;
        lock_animation(&mut transaction, account, id).await?;
        require_project(&mut transaction, account, to).await?;
        let query = sqlx::query_as::<_, AnimationRow>(MOVE_ANIMATION)
            .bind(id.uuid())
            .bind(to.uuid())
            .bind(at);
        let row = timed("move_animation", query.fetch_one(&mut *transaction)).await;
        let row = row.map_err(database)?;
        transaction.commit().await.map_err(database)?;
        row.into_record()
    }
}

/// Locks the animation's row until the transaction ends, and returns it.
pub(super) async fn lock_animation(
    connection: &mut PgConnection,
    account: AccountId,
    id: AnimationId,
) -> Result<AnimationRow, StoreError> {
    let query = sqlx::query_as::<_, AnimationRow>(LOCK_ANIMATION)
        .bind(id.uuid())
        .bind(account.uuid());
    let row = timed("lock_animation", query.fetch_optional(connection)).await;
    row.map_err(database)?.ok_or(StoreError::AnimationNotFound)
}

async fn insert_row(
    connection: &mut PgConnection,
    (account, new): (AccountId, &NewAnimationRecord),
    key: &Path,
) -> Result<AnimationRow, StoreError> {
    let bytes = u64::try_from(new.document.len()).unwrap_or(u64::MAX);
    let query = sqlx::query_as::<_, AnimationRow>(INSERT_ANIMATION)
        .bind(new.id.uuid())
        .bind(account.uuid())
        .bind(new.project.uuid())
        .bind(new.meta.title.as_str())
        .bind(i32::from(new.meta.width))
        .bind(i32::from(new.meta.height))
        .bind(i32::from(new.meta.frame_count))
        .bind(key.as_ref())
        .bind(bigint(bytes)?)
        .bind(FIRST_VERSION)
        .bind(new.at);
    let row = timed("insert_animation", query.fetch_one(connection)).await;
    row.map_err(database)
}

/// The `like` pattern of the titles that hold `text`, its `%`, `_` and escapes escaped.
fn contains_pattern(text: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_search_pattern_escapes_the_wildcards() {
        assert_eq!(contains_pattern("cat"), "%cat%");
        assert_eq!(contains_pattern("100%_off"), r"%100\%\_off%");
        assert_eq!(contains_pattern(r"a\b"), r"%a\\b%");
    }
}
