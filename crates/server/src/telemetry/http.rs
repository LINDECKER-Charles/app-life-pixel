//! Every request observed: a span named after its route template, the `http_*` metrics, and a
//! log line when it is answered.

use std::time::{Duration, Instant};

use axum::extract::{MatchedPath, Request};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use metrics::{counter, histogram};
use tracing::field::Empty;
use tracing::{Instrument, Span};

use super::metrics::{HTTP_REQUEST_DURATION_SECONDS, HTTP_REQUESTS_TOTAL};
use crate::http::request_id::RequestId;

/// The route of a request no route matched: never its raw path.
pub const UNMATCHED_ROUTE: &str = "unmatched";
/// The method label of a method outside HTTP's usual ones.
const OTHER_METHOD: &str = "OTHER";
/// Part of the templates axum gives the fallbacks of nested routers.
const FALLBACK_TEMPLATE_MARK: &str = "__private__axum";

/// What is known of a request once it is answered.
struct Answered {
    route: String,
    method: &'static str,
    status: StatusCode,
    elapsed: Duration,
}

/// Middleware: observes the request, under the request id the layer before it assigned.
pub async fn observe(request: Request, next: Next) -> Response {
    let route = route_label(&request);
    let method = method_label(request.method());
    let span = request_span(&request, &route, method);
    let started = Instant::now();
    let response = next.run(request).instrument(span.clone()).await;
    let answered = Answered {
        route,
        method,
        status: response.status(),
        elapsed: started.elapsed(),
    };
    log_answer(&span, &answered);
    record_metrics(answered);
    response
}

fn request_span(request: &Request, route: &str, method: &'static str) -> Span {
    let request_id = request
        .extensions()
        .get::<RequestId>()
        .map_or("", RequestId::as_str);
    tracing::info_span!(
        "request",
        otel.name = %format_args!("{method} {route}"),
        otel.kind = "server",
        otel.status_code = Empty,
        request_id,
        route,
        method,
        status = Empty,
        duration_ms = Empty,
    )
}

fn log_answer(span: &Span, answered: &Answered) {
    span.record("status", answered.status.as_u16());
    span.record("duration_ms", whole_milliseconds(answered.elapsed));
    if answered.status.is_server_error() {
        span.record("otel.status_code", "ERROR");
    }
    span.in_scope(|| tracing::info!("request answered"));
}

fn record_metrics(answered: Answered) {
    let Answered {
        route,
        method,
        status,
        elapsed,
    } = answered;
    let requests = [
        ("route", route.clone()),
        ("method", method.to_owned()),
        ("status_class", status_class(status).to_owned()),
    ];
    counter!(HTTP_REQUESTS_TOTAL, &requests).increment(1);
    let durations = [("route", route), ("method", method.to_owned())];
    histogram!(HTTP_REQUEST_DURATION_SECONDS, &durations).record(elapsed.as_secs_f64());
}

/// The route template the request matched, or `unmatched`.
fn route_label(request: &Request) -> String {
    request
        .extensions()
        .get::<MatchedPath>()
        .map(MatchedPath::as_str)
        .filter(|template| !template.contains(FALLBACK_TEMPLATE_MARK))
        .unwrap_or(UNMATCHED_ROUTE)
        .to_owned()
}

fn method_label(method: &Method) -> &'static str {
    match *method {
        Method::GET => "GET",
        Method::HEAD => "HEAD",
        Method::POST => "POST",
        Method::PUT => "PUT",
        Method::PATCH => "PATCH",
        Method::DELETE => "DELETE",
        Method::OPTIONS => "OPTIONS",
        _ => OTHER_METHOD,
    }
}

fn status_class(status: StatusCode) -> &'static str {
    match status.as_u16() / 100 {
        1 => "1xx",
        2 => "2xx",
        3 => "3xx",
        4 => "4xx",
        _ => "5xx",
    }
}

fn whole_milliseconds(elapsed: Duration) -> u64 {
    u64::try_from(elapsed.as_millis()).unwrap_or(u64::MAX)
}
