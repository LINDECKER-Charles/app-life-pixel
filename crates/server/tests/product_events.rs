//! Product events (H13), against a database of the local stack: the app's allow-list, the
//! subject an account keeps under one secret and loses under another, batching and the drop
//! counter, the purge, and the aggregates on a fixed set of events. Behind `stack-tests`.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

#[path = "common/mod.rs"]
mod common;

use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::connect_info::MockConnectInfo;
use axum::http::{Method, Request};
use life_pixel_server::config::{FromVariable, HmacKey};
use life_pixel_server::events::{self, PostgresEventSink};
use life_pixel_server::state::{AppState, Backends};
use life_pixel_server::testing::TestDatabase;
use life_pixel_server::{accounts, app, telemetry};
use life_pixel_service::AccountId;
use life_pixel_service::accounts::memory::RecordingMailer;
use life_pixel_service::events::subject;
use life_pixel_service::memory::InMemoryLibraryStore;
use life_pixel_service::ports::ProductEvent;
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool};
use tempfile::TempDir;
use time::{Duration as CalendarDuration, Month, OffsetDateTime, Time};

use common::auth::with_json;
use common::{
    Answer, Database, built_app, client_peer, empty, local_env, read_config, request, send,
};

/// How long a test waits for the sink's background task to write a batch.
const BATCH_WAIT: Duration = Duration::from_millis(1_500);
/// A second events secret, 64 hexadecimal characters, different from [`common::SECRET`].
const OTHER_SECRET: &str = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";

/// The server's state over a test database, wired with a real [`PostgresEventSink`] of its own.
struct EventsStack {
    database: TestDatabase,
    state: AppState,
    secret: HmacKey,
    _app_dir: TempDir,
}

impl EventsStack {
    /// A new test database, and the hosted ports over a [`PostgresEventSink`] of `secret`.
    async fn with_secret(secret: &str) -> Self {
        let database = TestDatabase::create().await.unwrap();
        let app_dir = built_app();
        let env = local_env(app_dir.path());
        let config = read_config(&env).unwrap();
        let secret = HmacKey::from_variable(secret).unwrap();
        let events = PostgresEventSink::spawn(database.pool().clone(), secret.clone());
        let backends = Backends {
            readiness: Arc::new(Database { answers: true }),
            library_store: Arc::new(InMemoryLibraryStore::new()),
            accounts: accounts::hosted_ports(
                database.pool(),
                Arc::new(RecordingMailer::new()),
                Arc::clone(&events),
            ),
            events,
        };
        let state = AppState::new(config, backends).unwrap();
        Self {
            database,
            state,
            secret,
            _app_dir: app_dir,
        }
    }

    /// A new test database with `common::SECRET`.
    async fn new() -> Self {
        Self::with_secret(common::SECRET).await
    }

    /// The answer of the public router to `request`.
    async fn send(&self, request: Request<Body>) -> Answer {
        let router = app::public_router(self.state.clone()).layer(MockConnectInfo(client_peer()));
        send(router, request).await
    }

    /// Every row of `product_events`, in the order they were written.
    async fn rows(&self) -> Vec<EventRow> {
        sqlx::query_as::<_, EventRow>(
            "select name, subject, properties, platform, app_version, language, occurred_at \
             from product_events order by occurred_at, id",
        )
        .fetch_all(self.database.pool())
        .await
        .unwrap()
    }
}

/// A row of `product_events`, read back for assertions.
#[derive(Clone, Debug, FromRow)]
struct EventRow {
    name: String,
    subject: Option<String>,
    properties: sqlx::types::Json<Value>,
    platform: Option<String>,
    app_version: Option<String>,
    language: Option<String>,
    occurred_at: OffsetDateTime,
}

/// A `POST /api/v1/events` of `body`, without a session.
fn post(body: &Value) -> Request<Body> {
    with_json(request(Method::POST, "/api/v1/events"), body)
}

/// A valid `export_completed` body: `format`, `bytes` and the app's context.
fn export_completed_body(format: &str, bytes: u64) -> Value {
    json!({
        "name": "export_completed",
        "properties": { "format": format, "bytes": bytes },
        "platform": "web",
        "appVersion": "1.0.0",
        "language": "fr",
    })
}

/// Bodies the allow-list refuses: a name outside it, and each way `export_completed`'s
/// properties can be wrong.
fn rejected_bodies() -> [Value; 4] {
    [
        json!({
            "name": "signed_up",
            "properties": { "format": "gif", "bytes": 1 },
            "platform": "web", "appVersion": "1.0.0", "language": "fr",
        }),
        json!({
            "name": "export_completed",
            "properties": { "format": "bmp", "bytes": 1 },
            "platform": "web", "appVersion": "1.0.0", "language": "fr",
        }),
        json!({
            "name": "export_completed",
            "properties": { "format": "gif" },
            "platform": "web", "appVersion": "1.0.0", "language": "fr",
        }),
        json!({
            "name": "export_completed",
            "properties": { "format": "gif", "bytes": -1 },
            "platform": "web", "appVersion": "1.0.0", "language": "fr",
        }),
    ]
}

/// Checks that `row` is the one `export_completed_body("gif", 8_421)` writes, without a session.
fn assert_export_row(row: &EventRow) {
    assert_eq!(row.name, "export_completed");
    assert_eq!(row.subject, None, "no session: no subject");
    assert_eq!(row.properties.0["format"], "gif");
    assert_eq!(row.properties.0["size"], "lt_10k");
    assert_eq!(row.properties.0["source"], "app");
    assert_eq!(row.platform.as_deref(), Some("web"));
    assert_eq!(row.app_version.as_deref(), Some("1.0.0"));
    assert_eq!(row.language.as_deref(), Some("fr"));
}

/// An `export_completed` event about `account`, with fixed properties.
fn sample_event(account: Option<AccountId>) -> ProductEvent {
    ProductEvent {
        name: "export_completed",
        account,
        properties: vec![
            ("format", "gif".to_owned()),
            ("size", "lt_1k".to_owned()),
            ("source", "app".to_owned()),
        ],
    }
}

/// How many of `rows` carry `expected` as their subject.
fn subject_count(rows: &[EventRow], expected: &str) -> usize {
    rows.iter()
        .filter(|row| row.subject.as_deref() == Some(expected))
        .count()
}

/// Writes one row of `product_events` directly, for the aggregates' fixed sets.
#[allow(clippy::too_many_arguments)] // The test names each column of the fixed row it writes.
async fn insert_event(
    pool: &PgPool,
    name: &'static str,
    subject: Option<&str>,
    properties: Value,
    occurred_at: OffsetDateTime,
) {
    sqlx::query(
        "insert into product_events (name, subject, properties, occurred_at) \
         values ($1, $2, $3, $4)",
    )
    .bind(name)
    .bind(subject)
    .bind(sqlx::types::Json(properties))
    .bind(occurred_at)
    .execute(pool)
    .await
    .unwrap();
}

/// Midnight UTC, `offset_days` after Monday 1 January 2024.
fn day(offset_days: i64) -> OffsetDateTime {
    let base = time::Date::from_calendar_date(2024, Month::January, 1).unwrap();
    (base + CalendarDuration::days(offset_days))
        .with_time(Time::MIDNIGHT)
        .assume_utc()
}

/// The fixed set of `signed_up` events [`signups_are_counted_per_day`] counts, plus a `signed_up`
/// outside the period and an event of another name, neither counted.
async fn insert_signups_fixture(pool: &PgPool) {
    insert_event(
        pool,
        "signed_up",
        None,
        json!({}),
        day(0) + CalendarDuration::hours(3),
    )
    .await;
    insert_event(
        pool,
        "signed_up",
        None,
        json!({}),
        day(0) + CalendarDuration::hours(4),
    )
    .await;
    insert_event(
        pool,
        "signed_up",
        None,
        json!({}),
        day(1) + CalendarDuration::hours(1),
    )
    .await;
    insert_event(pool, "signed_up", None, json!({}), day(-10)).await;
    insert_event(pool, "quota_rejected", None, json!({}), day(0)).await;
}

/// The fixed set of `export_completed` events [`active_subjects_are_distinct_per_day_week_and_month`]
/// counts: two subjects on the Monday, one on the Thursday, one the week after, and a null
/// subject that never counts.
async fn insert_active_subjects_fixture(pool: &PgPool) {
    insert_event(pool, "export_completed", Some("sub-1"), json!({}), day(0)).await;
    let noon = day(0) + CalendarDuration::hours(1);
    insert_event(pool, "export_completed", Some("sub-1"), json!({}), noon).await;
    insert_event(pool, "export_completed", Some("sub-2"), json!({}), day(0)).await;
    insert_event(pool, "export_completed", Some("sub-3"), json!({}), day(3)).await;
    insert_event(pool, "export_completed", Some("sub-1"), json!({}), day(7)).await;
    insert_event(pool, "export_completed", None, json!({}), day(0)).await;
}

/// The fixed set of `export_completed` events [`exports_are_counted_by_format_and_source`]
/// counts, and one outside the period.
async fn insert_exports_fixture(pool: &PgPool) {
    let rows: [(&str, &str, &str, i64); 5] = [
        ("sub-1", "gif", "app", 0),
        ("sub-2", "gif", "app", 0),
        ("sub-1", "gif", "mcp", 0),
        ("sub-1", "wasm", "mcp", 0),
        ("sub-1", "gif", "app", -10),
    ];
    for (subject, format, source, offset) in rows {
        let properties = json!({ "format": format, "source": source });
        insert_event(
            pool,
            "export_completed",
            Some(subject),
            properties,
            day(offset),
        )
        .await;
    }
}

/// The fixed set of `mcp_tool_called` events [`mcp_calls_are_counted_by_tool_and_outcome`] counts.
async fn insert_mcp_calls_fixture(pool: &PgPool) {
    let rows: [(&str, &str, &str); 4] = [
        ("sub-1", "export", "ok"),
        ("sub-2", "export", "ok"),
        ("sub-1", "export", "error"),
        ("sub-1", "list", "ok"),
    ];
    for (subject, tool, outcome) in rows {
        let properties = json!({ "tool": tool, "outcome": outcome });
        insert_event(pool, "mcp_tool_called", Some(subject), properties, day(0)).await;
    }
}

#[tokio::test]
async fn only_the_apps_allow_list_is_accepted() {
    let stack = EventsStack::new().await;
    let accepted = stack.send(post(&export_completed_body("gif", 8_421))).await;
    assert_eq!(accepted.status, 202, "{:?}", accepted.body);

    for body in rejected_bodies() {
        let answer = stack.send(post(&body)).await;
        answer.assert_problem(400, "request.malformed");
    }

    tokio::time::sleep(BATCH_WAIT).await;
    let rows = stack.rows().await;
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_export_row(&rows[0]);
}

#[tokio::test]
async fn the_subject_is_stable_per_account_and_differs_by_secret() {
    let stack = EventsStack::new().await;
    let account_a = stack.database.create_account().await.unwrap();
    let account_b = stack.database.create_account().await.unwrap();
    stack.state.events.record(sample_event(Some(account_a)));
    stack.state.events.record(sample_event(Some(account_a)));
    stack.state.events.record(sample_event(Some(account_b)));

    let other_secret = HmacKey::from_variable(OTHER_SECRET).unwrap();
    let other_sink = PostgresEventSink::spawn(stack.database.pool().clone(), other_secret.clone());
    other_sink.record(sample_event(Some(account_a)));
    tokio::time::sleep(BATCH_WAIT).await;

    let rows = stack.rows().await;
    assert_eq!(rows.len(), 4, "{rows:?}");
    let subject_a = subject(stack.secret.as_bytes(), account_a);
    let subject_b = subject(stack.secret.as_bytes(), account_b);
    let subject_a_other = subject(other_secret.as_bytes(), account_a);
    assert_ne!(subject_a, subject_b, "different accounts, same secret");
    assert_ne!(subject_a, subject_a_other, "same account, different secret");

    assert_eq!(subject_count(&rows, &subject_a), 2, "stable across events");
    assert_eq!(subject_count(&rows, &subject_b), 1);
    assert_eq!(subject_count(&rows, &subject_a_other), 1);
}

#[tokio::test]
async fn events_are_batched_and_overflow_is_dropped() {
    let handle = telemetry::recorder().unwrap_or_else(|error| panic!("recorder: {error}"));
    events::metrics::describe();
    let stack = EventsStack::new().await;

    // No `.await` between sends: the current-thread runtime never lets the sink's background
    // task run, so the channel fills to its bound and the next send is the one dropped.
    for _ in 0..10_001 {
        stack.state.events.record(sample_event(None));
    }

    tokio::time::sleep(Duration::from_secs(3)).await;
    let rows = stack.rows().await;
    assert_eq!(rows.len(), 10_000, "the channel holds at most 10,000");

    let answer = send(app::metrics_router(handle), empty(Method::GET, "/metrics")).await;
    let text = String::from_utf8_lossy(&answer.body).into_owned();
    let dropped = text
        .lines()
        .any(|line| line.starts_with("events_dropped_total 1"));
    assert!(dropped, "{text}");
}

#[tokio::test]
async fn old_events_are_purged() {
    let stack = EventsStack::new().await;
    let now = OffsetDateTime::now_utc();
    let old = now - CalendarDuration::days(events::RETENTION_DAYS + 1);
    let recent = now - CalendarDuration::days(events::RETENTION_DAYS - 1);
    insert_event(stack.database.pool(), "signed_up", None, json!({}), old).await;
    insert_event(stack.database.pool(), "signed_up", None, json!({}), recent).await;

    let deleted = events::purge(stack.database.pool()).await.unwrap();
    assert_eq!(deleted, 1);

    let rows = stack.rows().await;
    assert_eq!(rows.len(), 1);
    assert!(rows[0].occurred_at > old);
}

#[tokio::test]
async fn signups_are_counted_per_day() {
    let stack = EventsStack::new().await;
    let pool = stack.database.pool();
    insert_signups_fixture(pool).await;

    let period = events::queries::Period {
        since: day(0),
        until: day(2),
    };
    let counts = events::queries::signups_per_day(pool, period)
        .await
        .unwrap();
    let counts: Vec<(OffsetDateTime, i64)> = counts.into_iter().map(|c| (c.day, c.count)).collect();
    assert_eq!(counts, [(day(0), 2), (day(1), 1)]);
}

#[tokio::test]
async fn activation_counts_accounts_whose_first_export_falls_in_the_period() {
    let stack = EventsStack::new().await;
    let pool = stack.database.pool();
    // sub-1's first export falls in the period: activated. sub-2's first export is before it:
    // not activated in it. A null subject never activates an account.
    insert_event(pool, "export_completed", Some("sub-1"), json!({}), day(0)).await;
    insert_event(pool, "export_completed", Some("sub-1"), json!({}), day(5)).await;
    insert_event(pool, "export_completed", Some("sub-2"), json!({}), day(-3)).await;
    insert_event(pool, "export_completed", Some("sub-2"), json!({}), day(0)).await;
    insert_event(pool, "export_completed", None, json!({}), day(0)).await;

    let period = events::queries::Period {
        since: day(-1),
        until: day(1),
    };
    let activated = events::queries::activated_accounts(pool, period)
        .await
        .unwrap();
    assert_eq!(activated, 1);
}

#[tokio::test]
async fn active_subjects_are_distinct_per_day_week_and_month() {
    let stack = EventsStack::new().await;
    let pool = stack.database.pool();
    insert_active_subjects_fixture(pool).await;
    let period = events::queries::Period {
        since: day(0),
        until: day(8),
    };

    let per_day = events::queries::active_subjects_per_day(pool, period)
        .await
        .unwrap();
    let per_day: Vec<(OffsetDateTime, i64)> =
        per_day.into_iter().map(|c| (c.day, c.count)).collect();
    assert_eq!(per_day, [(day(0), 2), (day(3), 1), (day(7), 1)]);

    // day(0) and day(3) fall in the same ISO week: sub-1, sub-2 and sub-3 are all active in it.
    let per_week = events::queries::active_subjects_per_week(pool, period)
        .await
        .unwrap();
    let per_week: Vec<i64> = per_week.into_iter().map(|c| c.count).collect();
    assert_eq!(per_week, [3, 1]);

    let per_month = events::queries::active_subjects_per_month(pool, period)
        .await
        .unwrap();
    let per_month: Vec<i64> = per_month.into_iter().map(|c| c.count).collect();
    assert_eq!(per_month, [3]);
}

#[tokio::test]
async fn exports_are_counted_by_format_and_source() {
    let stack = EventsStack::new().await;
    let pool = stack.database.pool();
    insert_exports_fixture(pool).await;

    let period = events::queries::Period {
        since: day(-1),
        until: day(1),
    };
    let counts = events::queries::exports_by_format_and_source(pool, period)
        .await
        .unwrap();
    let counts: Vec<(String, String, i64)> = counts
        .into_iter()
        .map(|c| (c.format, c.source, c.count))
        .collect();
    assert_eq!(
        counts,
        [
            ("gif".to_owned(), "app".to_owned(), 2),
            ("gif".to_owned(), "mcp".to_owned(), 1),
            ("wasm".to_owned(), "mcp".to_owned(), 1),
        ]
    );
}

#[tokio::test]
async fn mcp_calls_are_counted_by_tool_and_outcome() {
    let stack = EventsStack::new().await;
    let pool = stack.database.pool();
    insert_mcp_calls_fixture(pool).await;

    let period = events::queries::Period {
        since: day(-1),
        until: day(1),
    };
    let counts = events::queries::mcp_calls_by_tool_and_outcome(pool, period)
        .await
        .unwrap();
    let counts: Vec<(String, String, i64)> = counts
        .into_iter()
        .map(|c| (c.tool, c.outcome, c.count))
        .collect();
    assert_eq!(
        counts,
        [
            ("export".to_owned(), "error".to_owned(), 1),
            ("export".to_owned(), "ok".to_owned(), 2),
            ("list".to_owned(), "ok".to_owned(), 1),
        ]
    );
}

#[tokio::test]
async fn storage_is_distributed_by_size_class() {
    let stack = EventsStack::new().await;
    let pool = stack.database.pool();
    let sizes = [500_i64, 5_000, 50_000, 500_000, 5_000_000];
    for bytes in sizes {
        let account = stack.database.create_account().await.unwrap();
        sqlx::query("update accounts set storage_used_bytes = $1 where id = $2")
            .bind(bytes)
            .bind(account.uuid())
            .execute(pool)
            .await
            .unwrap();
    }

    let counts = events::queries::storage_by_size_class(pool).await.unwrap();
    let counts: Vec<(String, i64)> = counts
        .into_iter()
        .map(|c| (c.size_class, c.count))
        .collect();
    assert_eq!(
        counts,
        [
            ("lt_1k".to_owned(), 1),
            ("lt_10k".to_owned(), 1),
            ("lt_100k".to_owned(), 1),
            ("lt_1m".to_owned(), 1),
            ("ge_1m".to_owned(), 1),
        ]
    );
}
