//! The product events' sink until H13 stores them: each is logged at debug level, then dropped.

use life_pixel_service::ports::{EventSink, ProductEvent};

/// Drops the product events, logging their names; H13 replaces it with the Postgres sink.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnrecordedEvents;

impl EventSink for UnrecordedEvents {
    fn record(&self, event: ProductEvent) {
        tracing::debug!(event = event.name, "a product event is not recorded yet");
    }
}
