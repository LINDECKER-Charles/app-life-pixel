//! `PostgresAdminSupportStore`: the support queue as the team reads it. The changes are in
//! `support_changes`.

use async_trait::async_trait;
use life_pixel_service::AccountId;
use life_pixel_service::admin::ports::{
    AdminMessage, AdminRequest, AdminStoreError, AdminSupportStore, AdminThread, QueueFilter,
    Recipient, RequestChange, TeamMessage,
};
use life_pixel_service::paging::{Cursor, Page, PageRequest};
use life_pixel_service::support::ports::SupportRequest;
use life_pixel_service::support::{
    Author, Category, SupportContext, SupportRequestId, SupportStatus,
};
use sqlx::{PgExecutor, PgPool, QueryBuilder};
use time::OffsetDateTime;
use uuid::Uuid;

use super::rows::{PagedRow, corrupt, database, page_of, push_page};
use crate::storage::metrics::timed;

/// The columns of a request's row, in [`RequestRow`]'s order, from `support_requests r` joined
/// to `accounts a`.
macro_rules! request_columns {
    () => {
        "r.id, r.category, r.status, r.screenshot_key is not null as has_screenshot, \
         r.created_at, r.updated_at, r.account_id, a.email::text as email, r.assigned_to, \
         r.first_response_at, r.resolved_at"
    };
}
pub(super) use request_columns;

const SELECT_QUEUE: &str = concat!(
    "select ",
    request_columns!(),
    " from support_requests r join accounts a on a.id = r.account_id where true"
);
const SELECT_REQUEST: &str = concat!(
    "select ",
    request_columns!(),
    " from support_requests r join accounts a on a.id = r.account_id where r.id = $1"
);
const SELECT_CONTEXT: &str = "select context::text from support_requests where id = $1";
const SELECT_MESSAGES: &str = "select id, author, admin_id, body, internal, created_at \
                               from support_messages where request_id = $1 \
                               order by created_at, id";
const SELECT_SCREENSHOT_KEY: &str = "select screenshot_key from support_requests where id = $1";

/// The support requests of the migrated database of a pool, as the team reads and answers them.
#[derive(Clone)]
pub struct PostgresAdminSupportStore {
    pub(super) pool: PgPool,
}

impl PostgresAdminSupportStore {
    /// The store over the migrated database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// A request's row, as the team sees it.
#[derive(sqlx::FromRow)]
pub(super) struct RequestRow {
    id: Uuid,
    category: String,
    status: String,
    has_screenshot: bool,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    account_id: Uuid,
    email: String,
    assigned_to: Option<String>,
    first_response_at: Option<OffsetDateTime>,
    resolved_at: Option<OffsetDateTime>,
}

impl RequestRow {
    /// The request of the row.
    pub(super) fn into_admin(self) -> Result<AdminRequest, AdminStoreError> {
        let category = Category::parse(&self.category)
            .map_err(|_| corrupt("support_requests.category", &self.category))?;
        let status = SupportStatus::parse(&self.status)
            .ok_or_else(|| corrupt("support_requests.status", &self.status))?;
        Ok(AdminRequest {
            request: SupportRequest {
                id: SupportRequestId::from_uuid(self.id),
                category,
                status,
                has_screenshot: self.has_screenshot,
                created_at: self.created_at,
                updated_at: self.updated_at,
            },
            account: AccountId::from_uuid(self.account_id),
            email: self.email,
            assigned_to: self.assigned_to,
            first_response_at: self.first_response_at,
            resolved_at: self.resolved_at,
        })
    }
}

impl PagedRow for RequestRow {
    type Record = AdminRequest;

    fn cursor(&self) -> Cursor {
        Cursor {
            updated_at: self.updated_at,
            id: self.id,
        }
    }

    fn into_record(self) -> Result<AdminRequest, AdminStoreError> {
        self.into_admin()
    }
}

/// A message's row, internal notes included.
#[derive(sqlx::FromRow)]
struct MessageRow {
    id: Uuid,
    author: String,
    admin_id: Option<String>,
    body: String,
    internal: bool,
    created_at: OffsetDateTime,
}

impl MessageRow {
    fn into_record(self) -> Result<AdminMessage, AdminStoreError> {
        let author = Author::parse(&self.author)
            .ok_or_else(|| corrupt("support_messages.author", &self.author))?;
        Ok(AdminMessage {
            id: self.id,
            author,
            admin_id: self.admin_id,
            body: self.body,
            is_internal: self.internal,
            created_at: self.created_at,
        })
    }
}

/// The request `id`, if it exists.
pub(super) async fn select_request(
    executor: impl PgExecutor<'_>,
    id: SupportRequestId,
) -> Result<Option<AdminRequest>, AdminStoreError> {
    let query = sqlx::query_as::<_, RequestRow>(SELECT_REQUEST).bind(id.uuid());
    let row = timed("admin_select_request", query.fetch_optional(executor)).await;
    row.map_err(database)?
        .map(RequestRow::into_admin)
        .transpose()
}

impl PostgresAdminSupportStore {
    async fn context(&self, id: SupportRequestId) -> Result<SupportContext, AdminStoreError> {
        let query = sqlx::query_scalar::<_, String>(SELECT_CONTEXT).bind(id.uuid());
        let json = timed("admin_select_context", query.fetch_one(&self.pool)).await;
        let json = json.map_err(database)?;
        SupportContext::parse(json.as_bytes())
            .map_err(|_| corrupt("support_requests.context", &json))
    }

    async fn messages(&self, id: SupportRequestId) -> Result<Vec<AdminMessage>, AdminStoreError> {
        let query = sqlx::query_as::<_, MessageRow>(SELECT_MESSAGES).bind(id.uuid());
        let rows = timed("admin_select_messages", query.fetch_all(&self.pool)).await;
        let messages = rows.map_err(database)?.into_iter();
        messages.map(MessageRow::into_record).collect()
    }
}

#[async_trait]
impl AdminSupportStore for PostgresAdminSupportStore {
    async fn queue(
        &self,
        filter: QueueFilter,
        page: PageRequest,
    ) -> Result<Page<AdminRequest>, AdminStoreError> {
        let mut builder = QueryBuilder::new(SELECT_QUEUE);
        if let Some(status) = filter.status {
            builder.push(" and r.status = ").push_bind(status.as_str());
        }
        if let Some(category) = filter.category {
            builder
                .push(" and r.category = ")
                .push_bind(category.as_str());
        }
        if let Some(assignee) = filter.assignee {
            builder.push(" and r.assigned_to = ").push_bind(assignee);
        }
        push_page(&mut builder, &page, ("r.updated_at", "r.id"));
        let query = builder.build_query_as::<RequestRow>();
        let rows = timed("admin_select_queue", query.fetch_all(&self.pool)).await;
        page_of(rows.map_err(database)?, &page)
    }

    async fn thread(&self, id: SupportRequestId) -> Result<Option<AdminThread>, AdminStoreError> {
        let Some(request) = select_request(&self.pool, id).await? else {
            return Ok(None);
        };
        Ok(Some(AdminThread {
            request,
            context: self.context(id).await?,
            messages: self.messages(id).await?,
        }))
    }

    async fn screenshot_key(
        &self,
        id: SupportRequestId,
    ) -> Result<Option<Option<String>>, AdminStoreError> {
        let query = sqlx::query_scalar::<_, Option<String>>(SELECT_SCREENSHOT_KEY).bind(id.uuid());
        let key = timed(
            "admin_select_screenshot_key",
            query.fetch_optional(&self.pool),
        )
        .await;
        key.map_err(database)
    }

    async fn update(&self, change: RequestChange) -> Result<Option<AdminRequest>, AdminStoreError> {
        self.change_request(change).await
    }

    async fn add_team_message(
        &self,
        message: TeamMessage,
    ) -> Result<Option<Recipient>, AdminStoreError> {
        self.insert_team_message(message).await
    }
}
