//! `PostgresSupportStore`: the `support_requests` and `support_messages` tables.

use async_trait::async_trait;
use life_pixel_service::AccountId;
use life_pixel_service::paging::{Page, PageRequest};
use life_pixel_service::support::ports::{
    NewSupportRequest, NewUserMessage, SupportMessage, SupportRequest, SupportStore,
    SupportStoreError, SupportThread, UserReply,
};
use life_pixel_service::support::{SupportRequestId, SupportStatus};
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

use super::rows::{MessageRow, RequestRow, database, request_columns};
use crate::storage::metrics::timed;

const INSERT_REQUEST: &str = "insert into support_requests (id, account_id, category, status, \
                              context, screenshot_key, created_at, updated_at) \
                              values ($1, $2, $3, $4, $5::jsonb, $6, $7, $8)";
const INSERT_MESSAGE: &str = "insert into support_messages (id, request_id, author, body, \
                              internal, created_at) values ($1, $2, $3, $4, false, $5)";
const SELECT_PAGE: &str = concat!(
    "select ",
    request_columns!(),
    " from support_requests where account_id = $1 \
     and ($2::timestamptz is null or (updated_at, id) < ($2, $3)) \
     order by updated_at desc, id desc limit $4"
);
const SELECT_REQUEST: &str = concat!(
    "select ",
    request_columns!(),
    " from support_requests where id = $1 and account_id = $2"
);
const SELECT_MESSAGES: &str = "select id, author, body, created_at from support_messages \
                               where request_id = $1 and not internal order by created_at, id";
const LOCK_STATUS: &str =
    "select status from support_requests where id = $1 and account_id = $2 for update";
const UPDATE_STATUS: &str =
    "update support_requests set status = $2, updated_at = $3 where id = $1";

/// The support requests of the migrated database of a pool.
#[derive(Clone)]
pub struct PostgresSupportStore {
    pool: PgPool,
}

impl PostgresSupportStore {
    /// The store over the migrated database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Inserts `message`, not internal, into the request `request`.
async fn insert_message(
    executor: impl PgExecutor<'_>,
    request: SupportRequestId,
    message: &SupportMessage,
) -> Result<(), SupportStoreError> {
    let query = sqlx::query(INSERT_MESSAGE)
        .bind(message.id)
        .bind(request.uuid())
        .bind(message.author.as_str())
        .bind(&message.body)
        .bind(message.created_at);
    timed("insert_support_message", query.execute(executor))
        .await
        .map_err(database)?;
    Ok(())
}

/// The status of the request `reply` answers, locked until the transaction ends; `None` when
/// its account has no such request.
async fn lock_status(
    executor: impl PgExecutor<'_>,
    reply: &NewUserMessage,
) -> Result<Option<SupportStatus>, SupportStoreError> {
    let query = sqlx::query_scalar::<_, String>(LOCK_STATUS)
        .bind(reply.request.uuid())
        .bind(reply.account.uuid());
    let status = timed("lock_support_request", query.fetch_optional(executor))
        .await
        .map_err(database)?;
    let Some(status) = status else {
        return Ok(None);
    };
    let parsed = SupportStatus::parse(&status);
    let corrupt = || SupportStoreError(format!("support_requests.status holds {status}"));
    parsed.map(Some).ok_or_else(corrupt)
}

/// Inserts the request of `new`, without its message.
async fn insert_request(
    executor: impl PgExecutor<'_>,
    new: &NewSupportRequest,
) -> Result<(), SupportStoreError> {
    let context = serde_json::to_string(&new.context)
        .map_err(|error| SupportStoreError(format!("context: {error}")))?;
    let request = &new.request;
    let query = sqlx::query(INSERT_REQUEST)
        .bind(request.id.uuid())
        .bind(new.account.uuid())
        .bind(request.category.as_str())
        .bind(request.status.as_str())
        .bind(context)
        .bind(new.screenshot_key.as_deref())
        .bind(request.created_at)
        .bind(request.updated_at);
    timed("insert_support_request", query.execute(executor))
        .await
        .map_err(database)?;
    Ok(())
}

#[async_trait]
impl SupportStore for PostgresSupportStore {
    async fn create(&self, new: NewSupportRequest) -> Result<(), SupportStoreError> {
        let mut transaction = self.pool.begin().await.map_err(database)?;
        insert_request(&mut *transaction, &new).await?;
        insert_message(&mut *transaction, new.request.id, &new.message).await?;
        transaction.commit().await.map_err(database)
    }

    async fn list(
        &self,
        account: AccountId,
        page: PageRequest,
    ) -> Result<Page<SupportRequest>, SupportStoreError> {
        let query = sqlx::query_as::<_, RequestRow>(SELECT_PAGE)
            .bind(account.uuid())
            .bind(page.cursor.map(|cursor| cursor.updated_at))
            .bind(page.cursor.map_or(Uuid::nil(), |cursor| cursor.id))
            .bind(i64::from(page.limit) + 1);
        let mut rows = timed("select_support_requests", query.fetch_all(&self.pool))
            .await
            .map_err(database)?;
        let has_more = rows.len() > usize::from(page.limit);
        rows.truncate(usize::from(page.limit));
        let next_cursor = rows.last().filter(|_| has_more).map(RequestRow::cursor);
        let items = rows.into_iter().map(RequestRow::into_record);
        Ok(Page {
            items: items.collect::<Result<_, _>>()?,
            next_cursor,
        })
    }

    async fn thread(
        &self,
        account: AccountId,
        id: SupportRequestId,
    ) -> Result<Option<SupportThread>, SupportStoreError> {
        let query = sqlx::query_as::<_, RequestRow>(SELECT_REQUEST)
            .bind(id.uuid())
            .bind(account.uuid());
        let row = timed("select_support_request", query.fetch_optional(&self.pool))
            .await
            .map_err(database)?;
        let Some(row) = row else { return Ok(None) };
        let query = sqlx::query_as::<_, MessageRow>(SELECT_MESSAGES).bind(id.uuid());
        let messages = timed("select_support_messages", query.fetch_all(&self.pool))
            .await
            .map_err(database)?;
        Ok(Some(SupportThread {
            request: row.into_record()?,
            messages: messages
                .into_iter()
                .map(MessageRow::into_record)
                .collect::<Result<_, _>>()?,
        }))
    }

    async fn add_user_message(
        &self,
        reply: NewUserMessage,
    ) -> Result<UserReply, SupportStoreError> {
        let mut transaction = self.pool.begin().await.map_err(database)?;
        let Some(current) = lock_status(&mut *transaction, &reply).await? else {
            return Ok(UserReply::NotFound);
        };
        let Some(next) = current.after_user_reply() else {
            return Ok(UserReply::Refused);
        };
        insert_message(&mut *transaction, reply.request, &reply.message).await?;
        let query = sqlx::query(UPDATE_STATUS)
            .bind(reply.request.uuid())
            .bind(next.as_str())
            .bind(reply.message.created_at);
        timed("update_support_status", query.execute(&mut *transaction))
            .await
            .map_err(database)?;
        transaction.commit().await.map_err(database)?;
        Ok(UserReply::Added { status: next })
    }
}
