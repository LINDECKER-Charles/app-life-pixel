//! Links to Grafana, where deep investigations stay: Explore split between a request's logs in
//! VictoriaLogs and its trace, found by the `request_id` its spans carry.

use reqwest::Url;
use serde_json::{Value, json};

use super::logs::quoted;
use super::time_range::TimeRange;
use crate::config::{GrafanaDatasources, ServiceUrl};

const EXPLORE_PATH: &str = "/explore";
/// The longest request id a link is made for.
const MAX_REQUEST_ID_CHARS: usize = 128;

/// What a Grafana link is made of.
#[derive(Clone, Debug)]
pub struct RequestLink<'a> {
    /// The environment's logs selector, without braces.
    pub logs_selector: &'a str,
    /// The request's id.
    pub request_id: &'a str,
    /// Where to look.
    pub range: TimeRange,
}

/// Whether `id` can be a request id: what the server assigns or keeps from a proxy — letters,
/// digits, `-`, `_`, `.` or `:`, 128 characters at most.
#[must_use]
pub fn is_request_id(id: &str) -> bool {
    let is_allowed = |byte: u8| byte.is_ascii_alphanumeric() || b"-_.:".contains(&byte);
    !id.is_empty() && id.len() <= MAX_REQUEST_ID_CHARS && id.bytes().all(is_allowed)
}

/// Grafana Explore at `grafana`, its left pane the logs of the request, its right pane the
/// request's trace.
#[must_use]
pub fn explore_url(
    grafana: &ServiceUrl,
    datasources: &GrafanaDatasources,
    link: &RequestLink<'_>,
) -> String {
    let panes = json!({
        "logs": logs_pane(&datasources.logs, link),
        "trace": trace_pane(&datasources.traces, link),
    });
    let base = grafana.join(EXPLORE_PATH);
    let mut url = Url::parse(&base).unwrap_or_else(|_| grafana.url().clone());
    url.query_pairs_mut()
        .append_pair("schemaVersion", "1")
        .append_pair("panes", &panes.to_string());
    url.to_string()
}

/// Explore's left pane: the request's log lines.
fn logs_pane(datasource: &str, link: &RequestLink<'_>) -> Value {
    let expr = format!(
        "{{{}}} AND request_id:{}",
        link.logs_selector,
        quoted(link.request_id)
    );
    json!({
        "datasource": datasource,
        "queries": [{ "refId": "A", "datasource": { "uid": datasource }, "expr": expr }],
        "range": pane_range(link.range),
    })
}

/// Explore's right pane: the trace whose spans carry the request id.
fn trace_pane(datasource: &str, link: &RequestLink<'_>) -> Value {
    json!({
        "datasource": datasource,
        "queries": [{
            "refId": "A",
            "datasource": { "uid": datasource },
            "queryType": "search",
            "tags": format!("request_id={}", link.request_id),
        }],
        "range": pane_range(link.range),
    })
}

/// A pane's range, in milliseconds since the epoch, as strings.
fn pane_range(range: TimeRange) -> Value {
    json!({
        "from": (range.from.unix_timestamp() * 1000).to_string(),
        "to": (range.to.unix_timestamp() * 1000).to_string(),
    })
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use super::*;
    use crate::config::FromVariable;

    #[test]
    fn a_link_opens_the_request_logs_and_its_trace() {
        let grafana = ServiceUrl::from_variable("https://grafana.example.org/").unwrap();
        let now = OffsetDateTime::from_unix_timestamp(1_800_000_000).unwrap();
        let range = TimeRange::parse(Some("now-1h"), None, now).unwrap();
        let link = RequestLink {
            logs_selector: "env=\"staging\"",
            request_id: "0190f6a2-7c1e",
            range,
        };
        let url = explore_url(&grafana, &GrafanaDatasources::default(), &link);
        let parsed = Url::parse(&url).unwrap();
        assert_eq!(parsed.path(), "/explore");
        let panes = parsed
            .query_pairs()
            .find(|(name, _)| name == "panes")
            .map(|(_, value)| serde_json::from_str::<serde_json::Value>(&value).unwrap())
            .unwrap();
        let logs = &panes["logs"]["queries"][0]["expr"];
        assert_eq!(logs, "{env=\"staging\"} AND request_id:\"0190f6a2-7c1e\"");
        assert_eq!(
            panes["trace"]["queries"][0]["tags"],
            "request_id=0190f6a2-7c1e"
        );
        assert_eq!(panes["logs"]["range"]["to"], "1800000000000");
    }

    #[test]
    fn only_request_ids_get_a_link() {
        assert!(is_request_id("0190f6a2-7c1e-7000-8000-000000000000"));
        for invalid in ["", "a b", "x\"y", &"a".repeat(129)] {
            assert!(!is_request_id(invalid), "{invalid}");
        }
    }
}
