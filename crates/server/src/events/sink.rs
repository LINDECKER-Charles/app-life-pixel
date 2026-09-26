//! `PostgresEventSink`: queues product events on a bounded channel so that [`EventSink::record`]
//! never blocks and never fails the caller, and writes them to Postgres by batches (H13).

use std::sync::Arc;
use std::time::Duration;

use life_pixel_service::events::subject;
use life_pixel_service::ports::{EventSink, ProductEvent};
use serde_json::{Map, Value};
use sqlx::{PgPool, QueryBuilder};
use time::OffsetDateTime;
use tokio::sync::mpsc;
use tokio::time::MissedTickBehavior;

use super::metrics::count_dropped;
use crate::config::HmacKey;

/// Events queued before [`EventSink::record`] starts dropping the next one.
const CHANNEL_CAPACITY: usize = 10_000;
/// The largest batch one insert writes.
const BATCH_MAX: usize = 100;
/// How long a batch waits for more events before it is written anyway.
const BATCH_INTERVAL: Duration = Duration::from_secs(1);

/// The property `POST /api/v1/events` carries the app's platform in; moved to its own column,
/// never part of the stored `properties`.
pub const PLATFORM_PROPERTY: &str = "platform";
/// The property carrying the app's version, moved to its own column.
pub const APP_VERSION_PROPERTY: &str = "app_version";
/// The property carrying the interface's language, moved to its own column.
pub const LANGUAGE_PROPERTY: &str = "language";

const INSERT_EVENTS: &str = "insert into product_events \
     (name, subject, properties, platform, app_version, language, occurred_at) ";

/// An event, timestamped when [`EventSink::record`] queued it.
struct Queued {
    event: ProductEvent,
    occurred_at: OffsetDateTime,
}

/// Batches product events to Postgres: a bounded channel of [`CHANNEL_CAPACITY`], drained by a
/// background task every [`BATCH_INTERVAL`] or as soon as [`BATCH_MAX`] events are queued.
pub struct PostgresEventSink {
    sender: mpsc::Sender<Queued>,
}

impl PostgresEventSink {
    /// Spawns the background task on the current runtime, pseudonymizing accounts with `secret`,
    /// and returns the sink the rest of the server records through.
    #[must_use]
    pub fn spawn(pool: PgPool, secret: HmacKey) -> Arc<dyn EventSink> {
        let (sender, receiver) = mpsc::channel(CHANNEL_CAPACITY);
        tokio::spawn(run(pool, secret, receiver));
        Arc::new(Self { sender })
    }
}

impl EventSink for PostgresEventSink {
    fn record(&self, event: ProductEvent) {
        let queued = Queued {
            event,
            occurred_at: OffsetDateTime::now_utc(),
        };
        if self.sender.try_send(queued).is_err() {
            count_dropped();
        }
    }
}

/// Drains `receiver` until every [`PostgresEventSink`] is dropped, writing a batch every
/// [`BATCH_INTERVAL`] or as soon as [`BATCH_MAX`] events are queued.
async fn run(pool: PgPool, secret: HmacKey, mut receiver: mpsc::Receiver<Queued>) {
    let mut buffer = Vec::with_capacity(BATCH_MAX);
    let mut interval = tokio::time::interval(BATCH_INTERVAL);
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            biased;
            received = receiver.recv() => {
                let Some(queued) = received else { break };
                buffer.push(queued);
                if buffer.len() >= BATCH_MAX {
                    flush(&pool, &secret, &mut buffer).await;
                }
            }
            _ = interval.tick() => flush(&pool, &secret, &mut buffer).await,
        }
    }
    flush(&pool, &secret, &mut buffer).await;
}

/// Writes `buffer` as one multi-row insert, then empties it; a failure is logged and the batch
/// dropped, never returned to the caller.
async fn flush(pool: &PgPool, secret: &HmacKey, buffer: &mut Vec<Queued>) {
    if buffer.is_empty() {
        return;
    }
    let mut builder = QueryBuilder::new(INSERT_EVENTS);
    builder.push_values(buffer.drain(..), |mut row, queued| {
        let bound = BoundEvent::of(secret, queued);
        row.push_bind(bound.name)
            .push_bind(bound.subject)
            .push_bind(sqlx::types::Json(bound.properties))
            .push_bind(bound.platform)
            .push_bind(bound.app_version)
            .push_bind(bound.language)
            .push_bind(bound.occurred_at);
    });
    log_failure(builder.build().execute(pool).await);
}

/// A queued event, ready for its row's bindings: the subject pseudonymized, the context split out
/// of its properties.
struct BoundEvent {
    name: &'static str,
    subject: Option<String>,
    properties: Value,
    platform: Option<String>,
    app_version: Option<String>,
    language: Option<String>,
    occurred_at: OffsetDateTime,
}

impl BoundEvent {
    fn of(secret: &HmacKey, queued: Queued) -> Self {
        let subject = queued
            .event
            .account
            .map(|account| subject(secret.as_bytes(), account));
        let (properties, platform, app_version, language) = split_context(queued.event.properties);
        Self {
            name: queued.event.name,
            subject,
            properties,
            platform,
            app_version,
            language,
            occurred_at: queued.occurred_at,
        }
    }
}

/// Logs a batch's failure; it is never returned to the caller.
fn log_failure(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>) {
    if let Err(error) = result {
        tracing::warn!(%error, "a batch of product events was not written");
    }
}

/// The app's context, pulled out of `properties`; what is left is exactly what
/// `docs/v1/service.md` documents for the event's name.
fn split_context(
    properties: Vec<(&'static str, String)>,
) -> (Value, Option<String>, Option<String>, Option<String>) {
    let mut platform = None;
    let mut app_version = None;
    let mut language = None;
    let mut rest = Map::new();
    for (key, value) in properties {
        match key {
            PLATFORM_PROPERTY => platform = Some(value),
            APP_VERSION_PROPERTY => app_version = Some(value),
            LANGUAGE_PROPERTY => language = Some(value),
            _ => {
                rest.insert(key.to_owned(), Value::String(value));
            }
        }
    }
    (Value::Object(rest), platform, app_version, language)
}
