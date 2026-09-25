//! Logs as JSON on stdout, one event per line, flat: `timestamp`, `level`, `target`, the fields
//! of the spans in scope — `request_id`, `route`, `method`, `status`, `duration_ms` — and the
//! event's own, `message` included.

use std::fmt;

use serde_json::{Map, Value};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use tracing::field::{Field, Visit};
use tracing::{Event, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::format::{JsonFields, Writer};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields, FormattedFields};
use tracing_subscriber::registry::{LookupSpan, SpanRef};

/// The prefix of the fields meant for traces only, such as `otel.name`.
const TRACE_ONLY_PREFIX: &str = "otel.";

/// The log layer: JSON lines on stdout.
pub fn layer<S>() -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .fmt_fields(JsonFields::new())
        .event_format(FlatJson)
        .with_writer(std::io::stdout)
}

/// The event format: one flat JSON object per line.
#[derive(Clone, Copy, Debug, Default)]
pub struct FlatJson;

impl<S, N> FormatEvent<S, N> for FlatJson
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'w> FormatFields<'w> + 'static,
{
    fn format_event(
        &self,
        context: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let metadata = event.metadata();
        let mut line = Map::new();
        line.insert("timestamp".to_owned(), Value::from(timestamp()));
        line.insert("level".to_owned(), Value::from(metadata.level().as_str()));
        line.insert("target".to_owned(), Value::from(metadata.target()));
        for span in context
            .event_scope()
            .into_iter()
            .flat_map(|scope| scope.from_root())
        {
            merge_span_fields::<S, N>(&span, &mut line);
        }
        event.record(&mut FieldVisitor(&mut line));
        writeln!(writer, "{}", Value::Object(line))
    }
}

fn timestamp() -> String {
    OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_default()
}

/// Adds the fields of `span`, as the JSON field formatter stored them, to `line`.
fn merge_span_fields<S, N>(span: &SpanRef<'_, S>, line: &mut Map<String, Value>)
where
    S: for<'a> LookupSpan<'a>,
    N: 'static,
{
    let extensions = span.extensions();
    let Some(fields) = extensions.get::<FormattedFields<N>>() else {
        return;
    };
    let Ok(fields) = serde_json::from_str::<Map<String, Value>>(fields.as_str()) else {
        return;
    };
    line.extend(
        fields
            .into_iter()
            .filter(|(name, _)| !name.starts_with(TRACE_ONLY_PREFIX)),
    );
}

/// Records an event's fields into a JSON object.
struct FieldVisitor<'a>(&'a mut Map<String, Value>);

impl FieldVisitor<'_> {
    fn insert(&mut self, field: &Field, value: Value) {
        self.0.insert(field.name().to_owned(), value);
    }
}

impl Visit for FieldVisitor<'_> {
    fn record_f64(&mut self, field: &Field, value: f64) {
        self.insert(field, Value::from(value));
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        self.insert(field, Value::from(value));
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        self.insert(field, Value::from(value));
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        self.insert(field, Value::from(value));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.insert(field, Value::from(value));
    }

    fn record_error(&mut self, field: &Field, value: &(dyn std::error::Error + 'static)) {
        self.insert(field, Value::from(value.to_string()));
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        self.insert(field, Value::from(format!("{value:?}")));
    }
}

#[cfg(test)]
mod tests {
    use std::io;
    use std::sync::{Arc, Mutex};

    use tracing_subscriber::fmt::MakeWriter;
    use tracing_subscriber::layer::SubscriberExt;

    use super::*;

    #[derive(Clone, Default)]
    struct Buffer(Arc<Mutex<Vec<u8>>>);

    impl io::Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for Buffer {
        type Writer = Self;

        fn make_writer(&'a self) -> Self {
            self.clone()
        }
    }

    #[test]
    fn an_event_is_one_flat_line_with_its_spans_fields() {
        let buffer = Buffer::default();
        let layer = tracing_subscriber::fmt::layer()
            .fmt_fields(JsonFields::new())
            .event_format(FlatJson)
            .with_writer(buffer.clone());
        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!(
                "request",
                otel.name = "GET /healthz",
                request_id = "0190",
                status = tracing::field::Empty
            );
            span.record("status", 200);
            span.in_scope(|| tracing::info!(duration_ms = 3, "request completed"));
        });
        let output = String::from_utf8(buffer.0.lock().unwrap().clone()).unwrap();
        assert_eq!(output.lines().count(), 1);
        let line: Map<String, Value> = serde_json::from_str(&output).unwrap();
        assert_eq!(line["level"], "INFO");
        assert_eq!(line["message"], "request completed");
        assert_eq!(line["request_id"], "0190");
        assert_eq!(line["status"], 200);
        assert_eq!(line["duration_ms"], 3);
        assert!(line.contains_key("timestamp") && line.contains_key("target"));
        assert!(!line.contains_key("otel.name"));
    }
}
