//! A sink that keeps the events it is given, for tests to read.

use std::sync::{Mutex, PoisonError};

use crate::ports::{EventSink, ProductEvent};

/// Keeps every recorded event, in order.
#[derive(Debug, Default)]
pub struct RecordingEvents {
    events: Mutex<Vec<ProductEvent>>,
}

impl RecordingEvents {
    /// A sink with no event yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The events recorded so far, in order.
    #[must_use]
    pub fn events(&self) -> Vec<ProductEvent> {
        self.events
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl EventSink for RecordingEvents {
    fn record(&self, event: ProductEvent) {
        let mut events = self.events.lock().unwrap_or_else(PoisonError::into_inner);
        events.push(event);
    }
}
