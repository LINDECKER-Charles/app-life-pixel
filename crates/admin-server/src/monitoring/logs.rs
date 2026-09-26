//! VictoriaLogs: `/select/logsql/query` with `{LOGS_SEL} AND level:i(<level>) AND "<q>"`, the
//! text escaped so that it stays one phrase, 500 lines at most.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::Deserialize;
use serde_json::{Map, Value};

use super::schema::LogLine;
use super::source::{Endpoint, Source, SourceError};
use super::time_range::{TimeRange, unix_seconds};
use crate::config::ServiceUrl;

/// The most lines a query returns.
pub const MAX_LOG_LINES: usize = 500;
/// The lines a query returns when the console sends no limit.
pub const DEFAULT_LOG_LINES: usize = 100;
const QUERY_PATH: &str = "/select/logsql/query";
const TIME_FIELD: &str = "_time";
const MESSAGE_FIELD: &str = "_msg";
const LEVEL_FIELD: &str = "level";
const REQUEST_ID_FIELD: &str = "request_id";
/// VictoriaLogs's own fields, beside `_time` and `_msg`, which a line does not repeat.
const INTERNAL_FIELDS: [&str; 2] = ["_stream", "_stream_id"];

/// A log level the console filters by.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// `error`.
    Error,
    /// `warn`.
    Warn,
    /// `info`.
    Info,
    /// `debug`.
    Debug,
    /// `trace`.
    Trace,
}

impl LogLevel {
    fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

/// What the console looks for.
#[derive(Clone, Debug, Default)]
pub struct LogSearch {
    /// Only this level, whatever its case.
    pub level: Option<LogLevel>,
    /// Only lines holding this text.
    pub text: Option<String>,
}

/// The LogsQL of `search` in the logs `selector` chooses — a stream filter's matchers, without
/// braces.
#[must_use]
pub fn logsql(selector: &str, search: &LogSearch) -> String {
    let mut query = format!("{{{selector}}}");
    if let Some(level) = search.level {
        let _infallible = write!(query, " AND {LEVEL_FIELD}:i({})", level.as_str());
    }
    if let Some(text) = search.text.as_deref().filter(|text| !text.is_empty()) {
        let _infallible = write!(query, " AND {}", quoted(text));
    }
    query
}

/// `text` as a LogsQL phrase: in double quotes, its backslashes, quotes and control characters
/// escaped, so that nothing in it reads as LogsQL.
#[must_use]
pub fn quoted(text: &str) -> String {
    let mut phrase = String::with_capacity(text.len() + 2);
    phrase.push('"');
    for character in text.chars() {
        match character {
            '"' => phrase.push_str("\\\""),
            '\\' => phrase.push_str("\\\\"),
            '\n' => phrase.push_str("\\n"),
            '\r' => phrase.push_str("\\r"),
            '\t' => phrase.push_str("\\t"),
            control if control.is_control() => {
                let _infallible = write!(phrase, "\\u{:04x}", u32::from(control));
            }
            other => phrase.push(other),
        }
    }
    phrase.push('"');
    phrase
}

/// A VictoriaLogs to query.
#[derive(Clone, Debug)]
pub struct LogsSource<'a> {
    /// The client.
    pub client: &'a reqwest::Client,
    /// `LPA_VICTORIALOGS_URL`.
    pub url: &'a ServiceUrl,
}

/// What a logs query asks.
#[derive(Clone, Debug)]
pub struct LogsRequest {
    /// The LogsQL, as [`logsql`] writes it.
    pub query: String,
    /// Where to look.
    pub range: TimeRange,
    /// The newest lines to return at most.
    pub limit: usize,
}

impl LogsSource<'_> {
    /// The newest lines `request` finds.
    ///
    /// # Errors
    ///
    /// [`SourceError::Unavailable`].
    pub async fn lines(&self, request: LogsRequest) -> Result<Vec<LogLine>, SourceError> {
        let LogsRequest {
            query,
            range,
            limit,
        } = request;
        let arguments = [
            ("query", query),
            ("start", unix_seconds(range.from)),
            ("end", unix_seconds(range.to)),
            ("limit", limit.to_string()),
        ];
        let endpoint = Endpoint {
            client: self.client,
            source: Source::VictoriaLogs,
            url: self.url.join(QUERY_PATH),
        };
        let text = endpoint.get_text(&arguments).await?;
        let mut lines = text
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str::<Map<String, Value>>(line).map(log_line))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| SourceError::unavailable(Source::VictoriaLogs, error))?;
        lines.sort_by(|a, b| b.time.cmp(&a.time));
        lines.truncate(limit);
        Ok(lines)
    }
}

/// A line of VictoriaLogs's answer, as the console reads it.
fn log_line(mut fields: Map<String, Value>) -> LogLine {
    let mut take = |name: &str| fields.remove(name).map(|value| text_of(&value));
    let time = take(TIME_FIELD).unwrap_or_default();
    let message = take(MESSAGE_FIELD).unwrap_or_default();
    let level = take(LEVEL_FIELD);
    let request_id = take(REQUEST_ID_FIELD);
    for internal in INTERNAL_FIELDS {
        fields.remove(internal);
    }
    let fields: BTreeMap<String, String> = fields
        .into_iter()
        .map(|(name, value)| (name, text_of(&value)))
        .collect();
    LogLine {
        time,
        level,
        message,
        request_id,
        fields,
    }
}

fn text_of(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEL: &str = "compose_project=\"life-pixel-staging\"";

    #[test]
    fn a_search_is_the_selector_then_the_level_then_the_phrase() {
        assert_eq!(logsql(SEL, &LogSearch::default()), format!("{{{SEL}}}"));
        let search = LogSearch {
            level: Some(LogLevel::Error),
            text: Some("export failed".to_owned()),
        };
        let expected = format!("{{{SEL}}} AND level:i(error) AND \"export failed\"");
        assert_eq!(logsql(SEL, &search), expected);
    }

    #[test]
    fn the_text_is_escaped_and_stays_one_phrase() {
        assert_eq!(quoted(r#"say "hi""#), r#""say \"hi\"""#);
        assert_eq!(quoted(r"C:\path"), r#""C:\\path""#);
        assert_eq!(quoted("a\nb\tc\u{7}"), r#""a\nb\tc\u0007""#);
        let injection = quoted(r#"" OR * OR ""#);
        assert_eq!(injection, r#""\" OR * OR \"""#);
        let unescaped_quotes = injection
            .char_indices()
            .filter(|(index, character)| {
                *character == '"' && (*index == 0 || !injection[..*index].ends_with('\\'))
            })
            .count();
        assert_eq!(unescaped_quotes, 2);
    }

    #[test]
    fn a_line_keeps_its_fields_but_not_victorialogs_own() {
        let raw = r#"{"_time":"2027-01-15T08:00:00Z","_msg":"request answered",
            "_stream":"{}","_stream_id":"x","level":"INFO","request_id":"0190","status":200}"#;
        let line = log_line(serde_json::from_str(raw).unwrap());
        assert_eq!(line.message, "request answered");
        assert_eq!(line.level.as_deref(), Some("INFO"));
        assert_eq!(line.request_id.as_deref(), Some("0190"));
        assert_eq!(
            line.fields.into_iter().collect::<Vec<_>>(),
            [("status".to_owned(), "200".to_owned())]
        );
    }
}
