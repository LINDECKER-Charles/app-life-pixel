//! What can go wrong reading a monitoring source, and the one way every source is asked: a `GET`
//! with a query, answered in time.

use std::time::Duration;

use thiserror::Error;

use crate::http::problem::{Problem, codes};

/// How long a source has to answer.
pub const SOURCE_TIMEOUT: Duration = Duration::from_secs(10);
/// How long a source has to accept a connection.
const SOURCE_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// A monitoring source, as problems name it in `source`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// VictoriaMetrics, through its query-only proxy.
    VictoriaMetrics,
    /// VictoriaLogs.
    VictoriaLogs,
    /// Alertmanager.
    Alertmanager,
    /// Grafana, for links.
    Grafana,
}

impl Source {
    /// The name the console reads in a problem's `source`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::VictoriaMetrics => "victoriametrics",
            Self::VictoriaLogs => "victorialogs",
            Self::Alertmanager => "alertmanager",
            Self::Grafana => "grafana",
        }
    }
}

/// Why a source gave nothing.
#[derive(Debug, Error)]
pub enum SourceError {
    /// Its URL, the environment's selector or the alerts' filter is left empty.
    #[error("{} is not configured", .0.as_str())]
    NotConfigured(Source),
    /// It did not answer, answered an error, or answered what does not parse.
    /// The detail is what went wrong, for the log.
    #[error("{name} failed: {1}", name = .0.as_str())]
    Unavailable(Source, String),
}

impl SourceError {
    /// `source` failed with `detail`.
    pub fn unavailable(source: Source, detail: impl ToString) -> Self {
        Self::Unavailable(source, detail.to_string())
    }
}

impl From<SourceError> for Problem {
    fn from(error: SourceError) -> Self {
        match error {
            SourceError::NotConfigured(source) => {
                Problem::new(codes::MONITORING_NOT_CONFIGURED).with_param("source", source.as_str())
            }
            SourceError::Unavailable(source, detail) => {
                tracing::warn!(source = source.as_str(), %detail, "a monitoring source failed");
                Problem::new(codes::MONITORING_UNAVAILABLE).with_param("source", source.as_str())
            }
        }
    }
}

/// A client for the sources: it follows no redirect, and gives up connecting after 5 seconds.
///
/// # Errors
///
/// When the TLS backend cannot start.
pub fn client() -> reqwest::Result<reqwest::Client> {
    reqwest::Client::builder()
        .connect_timeout(SOURCE_CONNECT_TIMEOUT)
        .redirect(reqwest::redirect::Policy::none())
        .build()
}

/// An endpoint of a source: where a `GET` goes, and which source a failure is blamed on.
#[derive(Clone, Debug)]
pub struct Endpoint<'a> {
    /// The client.
    pub client: &'a reqwest::Client,
    /// The source.
    pub source: Source,
    /// The endpoint's URL, without query.
    pub url: String,
}

impl Endpoint<'_> {
    /// The body of `GET url?query`, when the source answers a success.
    ///
    /// # Errors
    ///
    /// [`SourceError::Unavailable`].
    pub async fn get_text(&self, query: &[(&str, String)]) -> Result<String, SourceError> {
        let source = self.source;
        let answer = self
            .client
            .get(&self.url)
            .query(query)
            .timeout(SOURCE_TIMEOUT)
            .send()
            .await
            .map_err(|error| SourceError::unavailable(source, error))?;
        let status = answer.status();
        if !status.is_success() {
            return Err(SourceError::unavailable(source, format!("status {status}")));
        }
        answer
            .text()
            .await
            .map_err(|error| SourceError::unavailable(source, error))
    }

    /// The JSON body of `GET url?query`, parsed into `T`.
    ///
    /// # Errors
    ///
    /// [`SourceError::Unavailable`].
    pub async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        query: &[(&str, String)],
    ) -> Result<T, SourceError> {
        let text = self.get_text(query).await?;
        serde_json::from_str(&text).map_err(|error| SourceError::unavailable(self.source, error))
    }
}
