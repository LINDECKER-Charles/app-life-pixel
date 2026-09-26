//! The adapters the CLI provides to `service`, beyond the local library store itself.

use life_pixel_service::ports::{EventSink, ProductEvent};

/// Drops every product event: [`Owner::Local`](life_pixel_service::Owner::Local) sends none, but
/// the use cases still need a sink to hold.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoEvents;

impl EventSink for NoEvents {
    fn record(&self, _event: ProductEvent) {}
}
