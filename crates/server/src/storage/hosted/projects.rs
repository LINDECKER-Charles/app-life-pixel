//! Projects: create, read, list, rename, delete with their animations.

use life_pixel_core::Name;
use life_pixel_service::paging::{Page, PageRequest};
use life_pixel_service::ports::library_store::{ProjectRecord, StoreError};
use life_pixel_service::{AccountId, ProjectId};
use object_store::path::Path;
use sqlx::{PgConnection, QueryBuilder};
use time::OffsetDateTime;

use super::HostedLibraryStore;
use super::account::{add_usage, lock_usage};
use super::paging::{PagedRow, page_of, push_page};
use super::rows::{ProjectRow, database, project_columns};
use crate::storage::metrics::timed;

const INSERT_PROJECT: &str = "insert into projects (id, account_id, name, created_at, updated_at) \
                              values ($1, $2, $3, $4, $5)";
const SELECT_PROJECT: &str = concat!(
    "select ",
    project_columns!(),
    " from projects p where p.id = $1 and p.account_id = $2"
);
const SELECT_PROJECTS: &str = concat!(
    "select ",
    project_columns!(),
    " from projects p where p.account_id = "
);
const RENAME_PROJECT: &str = concat!(
    "update projects p set name = $3, updated_at = $4 where p.id = $1 and p.account_id = $2 \
     returning ",
    project_columns!()
);
const LOCK_PROJECT: &str = "select id from projects where id = $1 and account_id = $2 for update";
const DELETE_PROJECT_ANIMATIONS: &str = "delete from animations \
                                         where project_id = $1 and account_id = $2 \
                                         returning document_key, document_bytes";
const DELETE_PROJECT: &str = "delete from projects where id = $1";

impl HostedLibraryStore {
    pub(super) async fn insert_project(
        &self,
        account: AccountId,
        project: ProjectRecord,
    ) -> Result<(), StoreError> {
        let query = sqlx::query(INSERT_PROJECT)
            .bind(project.id.uuid())
            .bind(account.uuid())
            .bind(project.name.as_str())
            .bind(project.created_at)
            .bind(project.updated_at);
        timed("insert_project", query.execute(&self.pool))
            .await
            .map_err(database)?;
        Ok(())
    }

    pub(super) async fn select_project(
        &self,
        account: AccountId,
        id: ProjectId,
    ) -> Result<ProjectRecord, StoreError> {
        let query = sqlx::query_as::<_, ProjectRow>(SELECT_PROJECT)
            .bind(id.uuid())
            .bind(account.uuid());
        let row = timed("select_project", query.fetch_optional(&self.pool)).await;
        row.map_err(database)?
            .ok_or(StoreError::ProjectNotFound)?
            .into_record()
    }

    pub(super) async fn select_projects(
        &self,
        account: AccountId,
        request: PageRequest,
    ) -> Result<Page<ProjectRecord>, StoreError> {
        let mut builder = QueryBuilder::new(SELECT_PROJECTS);
        builder.push_bind(account.uuid());
        push_page(&mut builder, &request);
        let query = builder.build_query_as::<ProjectRow>();
        let rows = timed("list_projects", query.fetch_all(&self.pool)).await;
        page_of(rows.map_err(database)?, &request)
    }

    pub(super) async fn update_project_name(
        &self,
        (account, id): (AccountId, ProjectId),
        (name, at): (Name, OffsetDateTime),
    ) -> Result<ProjectRecord, StoreError> {
        let query = sqlx::query_as::<_, ProjectRow>(RENAME_PROJECT)
            .bind(id.uuid())
            .bind(account.uuid())
            .bind(name.as_str())
            .bind(at);
        let row = timed("rename_project", query.fetch_optional(&self.pool)).await;
        row.map_err(database)?
            .ok_or(StoreError::ProjectNotFound)?
            .into_record()
    }

    /// Deletes the project and its animations, frees their usage, then deletes their documents.
    pub(super) async fn remove_project(
        &self,
        account: AccountId,
        id: ProjectId,
    ) -> Result<(), StoreError> {
        let mut transaction = self.begin().await?;
        lock_usage(&mut transaction, account)
            .await?
            .ok_or(StoreError::ProjectNotFound)?;
        let keys = delete_project_rows(&mut transaction, account, id).await?;
        transaction.commit().await.map_err(database)?;
        self.delete_objects(keys).await;
        Ok(())
    }
}

/// Deletes the project's row and its animations' rows, and frees their bytes; returns the keys
/// of their documents.
async fn delete_project_rows(
    connection: &mut PgConnection,
    account: AccountId,
    id: ProjectId,
) -> Result<Vec<Path>, StoreError> {
    require_project(&mut *connection, account, id).await?;
    let animations = sqlx::query_as::<_, (String, i64)>(DELETE_PROJECT_ANIMATIONS)
        .bind(id.uuid())
        .bind(account.uuid());
    let deleted = timed(
        "delete_project_animations",
        animations.fetch_all(&mut *connection),
    )
    .await
    .map_err(database)?;
    let project = sqlx::query(DELETE_PROJECT).bind(id.uuid());
    timed("delete_project", project.execute(&mut *connection))
        .await
        .map_err(database)?;
    let freed: i64 = deleted.iter().map(|(_, bytes)| bytes).sum();
    add_usage(connection, account, -freed).await?;
    Ok(deleted
        .into_iter()
        .map(|(key, _)| Path::from(key))
        .collect())
}

/// Checks that the project `id` belongs to the account, and locks it until the transaction ends.
pub(super) async fn require_project(
    connection: &mut PgConnection,
    account: AccountId,
    id: ProjectId,
) -> Result<(), StoreError> {
    let query = sqlx::query(LOCK_PROJECT)
        .bind(id.uuid())
        .bind(account.uuid());
    let found = timed("lock_project", query.fetch_optional(connection)).await;
    found
        .map_err(database)?
        .ok_or(StoreError::ProjectNotFound)?;
    Ok(())
}
