//! What the team changes on a request, each change and its audit entry in one transaction, under
//! the request's lock: its status and assignee, and its messages.

use life_pixel_service::admin::ports::{
    AdminMessage, AdminRequest, AdminStoreError, Recipient, RequestChange, RequestState,
    TeamMessage,
};
use life_pixel_service::support::{SupportRequestId, SupportStatus};
use serde_json::{Value, json};
use sqlx::PgExecutor;
use time::OffsetDateTime;

use super::rows::{corrupt, database, insert_entry};
use super::support::{PostgresAdminSupportStore, select_request};
use crate::storage::metrics::timed;

const LOCK_REQUEST: &str = "select r.status, r.assigned_to, r.first_response_at, r.resolved_at, \
                            a.email::text as email, a.language from support_requests r \
                            join accounts a on a.id = r.account_id where r.id = $1 for update of r";
const UPDATE_STATE: &str = "update support_requests set status = $2, assigned_to = $3, \
                            first_response_at = $4, resolved_at = $5, \
                            updated_at = coalesce($6, updated_at) where id = $1";
const INSERT_MESSAGE: &str = "insert into support_messages (id, request_id, author, admin_id, \
                              body, internal, created_at) values ($1, $2, $3, $4, $5, $6, $7)";

/// A request's state and its author, locked.
#[derive(sqlx::FromRow)]
struct LockedRow {
    status: String,
    assigned_to: Option<String>,
    first_response_at: Option<OffsetDateTime>,
    resolved_at: Option<OffsetDateTime>,
    email: String,
    language: String,
}

impl LockedRow {
    fn state(&self) -> Result<RequestState, AdminStoreError> {
        let status = SupportStatus::parse(&self.status)
            .ok_or_else(|| corrupt("support_requests.status", &self.status))?;
        Ok(RequestState {
            status,
            assigned_to: self.assigned_to.clone(),
            first_response_at: self.first_response_at,
            resolved_at: self.resolved_at,
        })
    }
}

/// The request `id`'s state and author, locked until the transaction ends; `None` when it does
/// not exist.
async fn lock(
    executor: impl PgExecutor<'_>,
    id: SupportRequestId,
) -> Result<Option<LockedRow>, AdminStoreError> {
    let query = sqlx::query_as::<_, LockedRow>(LOCK_REQUEST).bind(id.uuid());
    let row = timed("admin_lock_request", query.fetch_optional(executor)).await;
    row.map_err(database)
}

/// Writes `state` to the request `id`; its `updated_at` becomes `updated_at`, when given.
async fn write_state(
    executor: impl PgExecutor<'_>,
    id: SupportRequestId,
    (state, updated_at): (&RequestState, Option<OffsetDateTime>),
) -> Result<(), AdminStoreError> {
    let query = sqlx::query(UPDATE_STATE)
        .bind(id.uuid())
        .bind(state.status.as_str())
        .bind(state.assigned_to.as_deref())
        .bind(state.first_response_at)
        .bind(state.resolved_at)
        .bind(updated_at);
    timed("admin_update_request", query.execute(executor))
        .await
        .map_err(database)?;
    Ok(())
}

/// Inserts `message` of the team into the request `id`.
async fn insert_message(
    executor: impl PgExecutor<'_>,
    id: SupportRequestId,
    message: &AdminMessage,
) -> Result<(), AdminStoreError> {
    let query = sqlx::query(INSERT_MESSAGE)
        .bind(message.id)
        .bind(id.uuid())
        .bind(message.author.as_str())
        .bind(message.admin_id.as_deref())
        .bind(&message.body)
        .bind(message.is_internal)
        .bind(message.created_at);
    timed("admin_insert_message", query.execute(executor))
        .await
        .map_err(database)?;
    Ok(())
}

/// `state` as an audit entry keeps it, with the message that changed it.
fn with_message(mut state: Value, message: &AdminMessage) -> Value {
    if let Value::Object(fields) = &mut state {
        fields.insert("messageId".to_owned(), json!(message.id));
        fields.insert("internal".to_owned(), json!(message.is_internal));
    }
    state
}

impl PostgresAdminSupportStore {
    /// Applies `change` under the request's lock, with its audit entry: the request after it.
    /// Only a new status moves the request's `updated_at`, which its author sees.
    pub(super) async fn change_request(
        &self,
        change: RequestChange,
    ) -> Result<Option<AdminRequest>, AdminStoreError> {
        let mut transaction = self.pool.begin().await.map_err(database)?;
        let Some(locked) = lock(&mut *transaction, change.id).await? else {
            return Ok(None);
        };
        let before = locked.state()?;
        let at = change.audit.at;
        let mut after = before.clone();
        if let Some(status) = change.status {
            after = after.with_status(status, at);
        }
        if let Some(assignee) = &change.assigned_to {
            after = after.with_assignee(assignee.as_ref());
        }
        let updated_at = (after.status != before.status).then_some(at);
        write_state(&mut *transaction, change.id, (&after, updated_at)).await?;
        let entry = change
            .audit
            .entry(Some(before.to_json()), Some(after.to_json()));
        insert_entry(&mut *transaction, &entry).await?;
        let request = select_request(&mut *transaction, change.id).await?;
        transaction.commit().await.map_err(database)?;
        Ok(request)
    }

    /// Adds the message of `new` under the request's lock, with its audit entry; a reply also
    /// answers the request. Whom to email it to.
    pub(super) async fn insert_team_message(
        &self,
        new: TeamMessage,
    ) -> Result<Option<Recipient>, AdminStoreError> {
        let mut transaction = self.pool.begin().await.map_err(database)?;
        let Some(locked) = lock(&mut *transaction, new.request).await? else {
            return Ok(None);
        };
        let before = locked.state()?;
        insert_message(&mut *transaction, new.request, &new.message).await?;
        let (before, after) = if new.message.is_internal {
            (None, with_message(json!({}), &new.message))
        } else {
            let at = new.message.created_at;
            let after = before.clone().answered(at);
            write_state(&mut *transaction, new.request, (&after, Some(at))).await?;
            (
                Some(before.to_json()),
                with_message(after.to_json(), &new.message),
            )
        };
        insert_entry(&mut *transaction, &new.audit.entry(before, Some(after))).await?;
        transaction.commit().await.map_err(database)?;
        Ok(Some(Recipient {
            email: locked.email,
            language: locked.language,
        }))
    }
}
