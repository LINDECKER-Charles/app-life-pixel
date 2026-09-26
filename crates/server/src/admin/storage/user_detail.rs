//! A user in detail: the language, the counts of the library, the product events by name, the
//! sessions and the support requests.

use life_pixel_service::AccountId;
use life_pixel_service::admin::ports::{AdminStoreError, EventCount, UserDetail, UserSession};
use life_pixel_service::events::subject;
use life_pixel_service::support::ports::SupportRequest;
use time::OffsetDateTime;

use super::rows::{corrupt, database};
use super::support::RequestRow;
use super::users::{PostgresAdminUserStore, select_user};
use crate::storage::metrics::timed;

const SELECT_LANGUAGE: &str = "select language from accounts where id = $1";
const COUNT_LIBRARY: &str = "select (select count(*) from projects where account_id = $1), \
                             (select count(*) from animations where account_id = $1)";
const COUNT_EVENTS: &str = "select name, count(*) from product_events \
                            where subject = $1 and occurred_at >= $2 group by name order by name";
const SELECT_SESSIONS: &str = "select created_at, last_seen_at, expires_at from sessions \
                               where account_id = $1 order by last_seen_at desc";
const SELECT_REQUESTS: &str = concat!(
    "select ",
    super::support::request_columns!(),
    " from support_requests r join accounts a on a.id = r.account_id \
     where r.account_id = $1 order by r.updated_at desc, r.id desc"
);

impl PostgresAdminUserStore {
    /// The account `id` in detail, with its product events since `events_since`.
    pub(super) async fn select_detail(
        &self,
        id: AccountId,
        events_since: OffsetDateTime,
    ) -> Result<Option<UserDetail>, AdminStoreError> {
        let Some(summary) = select_user(&self.pool, id).await? else {
            return Ok(None);
        };
        let (project_count, animation_count) = self.library_counts(id).await?;
        Ok(Some(UserDetail {
            summary,
            language: self.language(id).await?,
            project_count,
            animation_count,
            events: self.events(id, events_since).await?,
            sessions: self.sessions(id).await?,
            support_requests: self.support_requests(id).await?,
        }))
    }

    async fn language(&self, id: AccountId) -> Result<String, AdminStoreError> {
        let query = sqlx::query_scalar::<_, String>(SELECT_LANGUAGE).bind(id.uuid());
        let language = timed("admin_select_language", query.fetch_one(&self.pool)).await;
        language.map_err(database)
    }

    async fn library_counts(&self, id: AccountId) -> Result<(u64, u64), AdminStoreError> {
        let query = sqlx::query_as::<_, (i64, i64)>(COUNT_LIBRARY).bind(id.uuid());
        let counts = timed("admin_count_library", query.fetch_one(&self.pool)).await;
        let (projects, animations) = counts.map_err(database)?;
        Ok((count(projects)?, count(animations)?))
    }

    async fn events(
        &self,
        id: AccountId,
        since: OffsetDateTime,
    ) -> Result<Vec<EventCount>, AdminStoreError> {
        let subject = subject(self.events_secret.as_bytes(), id);
        let query = sqlx::query_as::<_, (String, i64)>(COUNT_EVENTS)
            .bind(subject)
            .bind(since);
        let rows = timed("admin_count_events", query.fetch_all(&self.pool)).await;
        let rows = rows.map_err(database)?;
        let counts = rows.into_iter().map(|(name, events)| {
            let count = count(events)?;
            Ok(EventCount { name, count })
        });
        counts.collect()
    }

    async fn sessions(&self, id: AccountId) -> Result<Vec<UserSession>, AdminStoreError> {
        type SessionRow = (OffsetDateTime, OffsetDateTime, OffsetDateTime);
        let query = sqlx::query_as::<_, SessionRow>(SELECT_SESSIONS).bind(id.uuid());
        let rows = timed("admin_select_sessions", query.fetch_all(&self.pool)).await;
        let sessions = rows.map_err(database)?.into_iter();
        let sessions = sessions.map(|(created_at, last_seen_at, expires_at)| UserSession {
            created_at,
            last_seen_at,
            expires_at,
        });
        Ok(sessions.collect())
    }

    async fn support_requests(
        &self,
        id: AccountId,
    ) -> Result<Vec<SupportRequest>, AdminStoreError> {
        let query = sqlx::query_as::<_, RequestRow>(SELECT_REQUESTS).bind(id.uuid());
        let rows = timed("admin_select_user_requests", query.fetch_all(&self.pool)).await;
        let requests = rows.map_err(database)?.into_iter();
        let requests = requests.map(|row| row.into_admin().map(|request| request.request));
        requests.collect()
    }
}

/// A count, which is never negative.
fn count(value: i64) -> Result<u64, AdminStoreError> {
    u64::try_from(value).map_err(|_| corrupt("count", &value))
}
