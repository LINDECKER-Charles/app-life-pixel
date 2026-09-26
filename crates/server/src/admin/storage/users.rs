//! `PostgresAdminUserStore`: the accounts as the admin searches and reads them. Their details
//! are in `user_detail`, the changes in `user_changes`.

use async_trait::async_trait;
use life_pixel_service::AccountId;
use life_pixel_service::accounts::ports::AccountStatus;
use life_pixel_service::admin::ports::{
    AdminStoreError, AdminUserStore, Erasure, StatusChange, UserDetail, UserFilter, UserSummary,
};
use life_pixel_service::paging::{Cursor, Page, PageRequest};
use sqlx::{PgExecutor, PgPool, QueryBuilder};
use time::OffsetDateTime;
use uuid::Uuid;

use super::rows::{LIKE_ESCAPE, PagedRow, contains_pattern, corrupt, database, page_of, push_page};
use crate::config::HmacKey;
use crate::storage::metrics::timed;

/// The columns of a user's row, in [`UserRow`]'s order, from `accounts a`.
macro_rules! user_columns {
    () => {
        "a.id, a.email::text as email, a.email_verified_at is not null as is_email_verified, \
         a.plan, a.status, a.storage_used_bytes, a.created_at, \
         (select max(s.last_seen_at) from sessions s where s.account_id = a.id) as last_seen_at"
    };
}

const SEARCH_USERS: &str = concat!("select ", user_columns!(), " from accounts a where true");
const SELECT_USER: &str = concat!(
    "select ",
    user_columns!(),
    " from accounts a where a.id = $1"
);

/// The accounts of the migrated database of a pool, as the admin reads and changes them; the
/// product events of an account are found through its subject, keyed with `LP_EVENTS_SECRET`.
#[derive(Clone)]
pub struct PostgresAdminUserStore {
    pub(super) pool: PgPool,
    pub(super) events_secret: HmacKey,
}

impl PostgresAdminUserStore {
    /// The store over the migrated database of `pool`, with the key of the events' subjects.
    #[must_use]
    pub fn new(pool: PgPool, events_secret: HmacKey) -> Self {
        Self {
            pool,
            events_secret,
        }
    }
}

/// A user's row.
#[derive(sqlx::FromRow)]
pub(super) struct UserRow {
    id: Uuid,
    email: String,
    is_email_verified: bool,
    plan: String,
    status: String,
    storage_used_bytes: i64,
    created_at: OffsetDateTime,
    last_seen_at: Option<OffsetDateTime>,
}

impl PagedRow for UserRow {
    type Record = UserSummary;

    fn cursor(&self) -> Cursor {
        Cursor {
            updated_at: self.created_at,
            id: self.id,
        }
    }

    fn into_record(self) -> Result<UserSummary, AdminStoreError> {
        let status = AccountStatus::parse(&self.status)
            .ok_or_else(|| corrupt("accounts.status", &self.status))?;
        let used = u64::try_from(self.storage_used_bytes)
            .map_err(|_| corrupt("accounts.storage_used_bytes", &self.storage_used_bytes))?;
        Ok(UserSummary {
            id: AccountId::from_uuid(self.id),
            email: self.email,
            is_email_verified: self.is_email_verified,
            plan: self.plan,
            status,
            storage_used_bytes: used,
            created_at: self.created_at,
            last_seen_at: self.last_seen_at,
        })
    }
}

/// The account `id`, if it exists.
pub(super) async fn select_user(
    executor: impl PgExecutor<'_>,
    id: AccountId,
) -> Result<Option<UserSummary>, AdminStoreError> {
    let query = sqlx::query_as::<_, UserRow>(SELECT_USER).bind(id.uuid());
    let row = timed("admin_select_user", query.fetch_optional(executor)).await;
    row.map_err(database)?.map(UserRow::into_record).transpose()
}

#[async_trait]
impl AdminUserStore for PostgresAdminUserStore {
    async fn search(
        &self,
        filter: UserFilter,
        page: PageRequest,
    ) -> Result<Page<UserSummary>, AdminStoreError> {
        let mut builder = QueryBuilder::new(SEARCH_USERS);
        if let Some(query) = &filter.query {
            builder.push(" and (a.email ilike ");
            builder.push_bind(contains_pattern(query));
            builder.push(format_args!(" escape '{LIKE_ESCAPE}' or a.id = "));
            builder.push_bind(Uuid::parse_str(query.trim()).ok());
            builder.push(")");
        }
        if let Some(status) = filter.status {
            builder.push(" and a.status = ").push_bind(status.as_str());
        }
        push_page(&mut builder, &page, ("a.created_at", "a.id"));
        let query = builder.build_query_as::<UserRow>();
        let rows = timed("admin_search_users", query.fetch_all(&self.pool)).await;
        page_of(rows.map_err(database)?, &page)
    }

    async fn find(&self, id: AccountId) -> Result<Option<UserSummary>, AdminStoreError> {
        select_user(&self.pool, id).await
    }

    async fn detail(
        &self,
        id: AccountId,
        events_since: OffsetDateTime,
    ) -> Result<Option<UserDetail>, AdminStoreError> {
        self.select_detail(id, events_since).await
    }

    async fn set_status(&self, change: StatusChange) -> Result<bool, AdminStoreError> {
        self.change_status(change).await
    }

    async fn erase(&self, erasure: Erasure) -> Result<bool, AdminStoreError> {
        self.delete_account(erasure).await
    }
}
