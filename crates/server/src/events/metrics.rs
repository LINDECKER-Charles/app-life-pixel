//! `events_dropped_total`: product events dropped because the channel to Postgres was full.

use metrics::{counter, describe_counter};

/// Product events dropped, because [`super::PostgresEventSink`]'s channel was full.
pub const EVENTS_DROPPED_TOTAL: &str = "events_dropped_total";

/// Describes the events' metrics to the recorder, once it is installed.
pub fn describe() {
    describe_counter!(
        EVENTS_DROPPED_TOTAL,
        "Product events dropped: the channel to Postgres was full"
    );
}

/// Counts one dropped event.
pub(super) fn count_dropped() {
    counter!(EVENTS_DROPPED_TOTAL).increment(1);
}
