//! Telemetry: JSON logs on stdout filtered by `RUST_LOG`, OTLP traces when
//! `OTEL_EXPORTER_OTLP_ENDPOINT` is set, Prometheus metrics on the metrics listener. Never an
//! email address, a token, a password or a document in any of them.

mod http;
mod json_log;
mod metrics;
mod traces;

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::ExporterBuildError;
use opentelemetry_sdk::trace::SdkTracerProvider;
use thiserror::Error;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::{SubscriberInitExt, TryInitError};

use crate::config::Config;

pub use http::{UNMATCHED_ROUTE, observe};
pub use json_log::FlatJson;
pub use metrics::{
    HTTP_REQUEST_DURATION_SECONDS, HTTP_REQUESTS_TOTAL, METRICS_UPKEEP_PERIOD, recorder,
    render_metrics,
};

/// The service's name in traces.
pub const SERVICE_NAME: &str = "life-pixel-server";

/// Why telemetry could not start.
#[derive(Debug, Error)]
pub enum TelemetryError {
    /// `RUST_LOG` does not parse.
    #[error("RUST_LOG is invalid")]
    Filter,
    /// The OTLP exporter cannot be built from `OTEL_EXPORTER_OTLP_ENDPOINT`.
    #[error("OTEL_EXPORTER_OTLP_ENDPOINT is invalid: {0}")]
    Exporter(#[from] ExporterBuildError),
    /// Another subscriber was installed first.
    #[error("the log subscriber is already installed: {0}")]
    Subscriber(#[from] TryInitError),
}

/// The running telemetry: flushes the traces still buffered when shut down.
pub struct Telemetry {
    tracer_provider: Option<SdkTracerProvider>,
}

impl Telemetry {
    /// Installs the process's log subscriber, and its tracer when an OTLP endpoint is set.
    ///
    /// # Errors
    ///
    /// When the log filter or the endpoint is invalid, or a subscriber is already installed.
    pub fn init(config: &Config) -> Result<Self, TelemetryError> {
        let filter =
            EnvFilter::try_new(config.log_filter.as_str()).map_err(|_| TelemetryError::Filter)?;
        let tracer_provider = config
            .otlp_endpoint
            .as_deref()
            .map(|endpoint| traces::tracer_provider(endpoint, config.environment))
            .transpose()?;
        let traces_layer = tracer_provider.as_ref().map(|provider| {
            tracing_opentelemetry::layer().with_tracer(provider.tracer(SERVICE_NAME))
        });
        tracing_subscriber::registry()
            .with(filter)
            .with(json_log::layer())
            .with(traces_layer)
            .try_init()?;
        Ok(Self { tracer_provider })
    }

    /// Sends the spans still buffered, then stops the exporter.
    pub fn shutdown(self) {
        let Some(provider) = self.tracer_provider else {
            return;
        };
        if let Err(error) = provider.shutdown() {
            tracing::warn!(%error, "the traces could not all be sent");
        }
    }
}
