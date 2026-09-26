//! Monitoring: health, metrics, logs and alerts of the environments of `LPA_ENVIRONMENTS`, read
//! from the VPS's observability stack — VictoriaMetrics through a query-only proxy,
//! VictoriaLogs, Alertmanager — and links to Grafana. It only reads, and the console never sends
//! a query: it names a panel, and the admin server writes the query with the environment's
//! selector. A source left empty answers `monitoring.not_configured`, one that fails
//! `monitoring.unavailable`.

pub mod alerts;
pub mod links;
pub mod logs;
pub mod metrics;
pub mod panels;
pub mod routes;
pub mod schema;
pub mod source;
pub mod time_range;

use std::sync::Arc;

use futures_util::future::try_join_all;
use time::Duration;

use crate::config::{MonitoringConfig, Selectors, ServiceUrl};
use crate::http::problem::{Problem, codes};
use metrics::{MetricsSource, Window};
use panels::{Panel, SelectorKind};
use schema::{Overview, PanelSeries, Series, Tile};
use source::{Source, SourceError};
use time_range::{TimeRange, promql_duration};

/// A panel's series to read: over `range`, a point every `step`.
#[derive(Clone, Copy, Debug)]
pub struct PanelRequest {
    /// The panel.
    pub panel: Panel,
    /// The period.
    pub range: TimeRange,
    /// Between two points.
    pub step: Duration,
}

/// The monitoring sources and the environments they show.
#[derive(Clone, Debug)]
pub struct Monitoring {
    config: Arc<MonitoringConfig>,
    client: reqwest::Client,
}

impl Monitoring {
    /// The monitoring of `config`, asked through `client`.
    #[must_use]
    pub fn new(config: MonitoringConfig, client: reqwest::Client) -> Self {
        Self {
            config: Arc::new(config),
            client,
        }
    }

    /// The overview of `environment` over `range`: each tile's value at its end and one range
    /// earlier, asked all at once.
    ///
    /// # Errors
    ///
    /// `request.malformed` for an unknown environment, the problems of [`SourceError`].
    pub async fn overview(&self, environment: &str, range: TimeRange) -> Result<Overview, Problem> {
        let selectors = self.selectors(environment)?;
        let source = self.metrics()?;
        let window = promql_duration(range.length());
        let tiles = Panel::TILES.iter().map(|&panel| {
            let (source, window) = (&source, &window);
            async move {
                let selector = selector(selectors, panel.selector_kind())?;
                let promql = panel.queries()[0].promql(selector, window);
                let value = source.value_at(&promql, range.to).await?;
                let previous = source.value_at(&promql, range.from).await?;
                Ok::<_, SourceError>(Tile {
                    panel,
                    value,
                    previous,
                })
            }
        });
        Ok(Overview {
            environment: environment.to_owned(),
            from: range.from.unix_timestamp(),
            to: range.to.unix_timestamp(),
            tiles: try_join_all(tiles).await?,
        })
    }

    /// The series of a panel in `environment`, and the previous period's laid over them.
    ///
    /// # Errors
    ///
    /// `request.malformed` for an unknown environment, the problems of [`SourceError`].
    pub async fn series(
        &self,
        environment: &str,
        request: PanelRequest,
    ) -> Result<PanelSeries, Problem> {
        let PanelRequest { panel, range, step } = request;
        let selectors = self.selectors(environment)?;
        let source = self.metrics()?;
        let selector = selector(selectors, panel.selector_kind())?;
        let window = promql_duration(step);
        let queries: Vec<(&str, String)> = panel
            .queries()
            .iter()
            .map(|query| (query.name, query.promql(selector, &window)))
            .collect();
        let asked = windows(range, step).into_iter().flat_map(|window| {
            let source = &source;
            queries
                .iter()
                .map(move |(name, promql)| source.series((name, promql), window))
        });
        let mut answers = try_join_all(asked).await?.into_iter();
        let current: Vec<Series> = answers.by_ref().take(queries.len()).flatten().collect();
        Ok(PanelSeries {
            panel,
            environment: environment.to_owned(),
            from: range.from.unix_timestamp(),
            to: range.to.unix_timestamp(),
            step_seconds: step.whole_seconds(),
            series: current,
            previous: answers.flatten().collect(),
        })
    }

    /// The selectors of `environment`, one of `LPA_ENVIRONMENTS`.
    ///
    /// # Errors
    ///
    /// `request.malformed` for another environment.
    fn selectors(&self, environment: &str) -> Result<&Selectors, Problem> {
        self.config
            .selectors
            .get(environment)
            .ok_or_else(|| Problem::new(codes::REQUEST_MALFORMED))
    }

    fn metrics(&self) -> Result<MetricsSource<'_>, SourceError> {
        let url = configured(
            self.config.victoria_metrics.as_ref(),
            Source::VictoriaMetrics,
        )?;
        Ok(MetricsSource {
            client: &self.client,
            url,
        })
    }

    /// The configuration of the sources.
    #[must_use]
    pub fn config(&self) -> &MonitoringConfig {
        &self.config
    }

    /// The client the sources are asked through.
    #[must_use]
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

/// The source's URL, when configured.
///
/// # Errors
///
/// [`SourceError::NotConfigured`].
pub fn configured(url: Option<&ServiceUrl>, source: Source) -> Result<&ServiceUrl, SourceError> {
    url.ok_or(SourceError::NotConfigured(source))
}

/// The windows a panel's series are read over: the range, then the previous period, moved
/// onto the range.
fn windows(range: TimeRange, step: Duration) -> [Window; 2] {
    let current = Window {
        range,
        step,
        shift: Duration::ZERO,
    };
    let previous = Window {
        range: range.previous(),
        step,
        shift: range.length(),
    };
    [current, previous]
}

/// The selector of `kind` among `selectors`: without it, a query would read every environment.
fn selector(selectors: &Selectors, kind: SelectorKind) -> Result<&str, SourceError> {
    let found = match kind {
        SelectorKind::Services => selectors.metrics.as_deref(),
        SelectorKind::Containers => selectors.containers.as_deref(),
    };
    found.ok_or(SourceError::NotConfigured(Source::VictoriaMetrics))
}
