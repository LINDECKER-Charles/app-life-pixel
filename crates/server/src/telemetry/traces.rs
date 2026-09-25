//! Traces: OTLP over HTTP to `OTEL_EXPORTER_OTLP_ENDPOINT`, as service `life-pixel-server` in
//! its `deployment.environment`.

use opentelemetry::KeyValue;
use opentelemetry_otlp::{ExporterBuildError, Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;

use super::SERVICE_NAME;
use crate::config::Environment;

/// The path of the traces under an OTLP/HTTP endpoint.
const TRACES_PATH: &str = "/v1/traces";
const ENVIRONMENT_ATTRIBUTE: &str = "deployment.environment";

/// A provider that batches spans and sends them to `endpoint`, such as
/// `http://otel-collector:4318`.
///
/// # Errors
///
/// When the exporter cannot be built, for an invalid endpoint.
pub fn tracer_provider(
    endpoint: &str,
    environment: Environment,
) -> Result<SdkTracerProvider, ExporterBuildError> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .with_endpoint(format!("{}{TRACES_PATH}", endpoint.trim_end_matches('/')))
        .build()?;
    let resource = Resource::builder()
        .with_service_name(SERVICE_NAME)
        .with_attribute(KeyValue::new(ENVIRONMENT_ATTRIBUTE, environment.as_str()))
        .build();
    Ok(SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build())
}
