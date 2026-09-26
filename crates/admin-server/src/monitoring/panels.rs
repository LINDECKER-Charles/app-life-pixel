//! The panels: the console names one, the admin server writes its PromQL. `{SEL}` stands for the
//! environment's selector — the containers' for `memory` and `cpu` —, `$range` for the period a
//! tile covers or the step of a series, so that a series' points add up to its tile.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Where `{SEL}` goes in a template.
const SELECTOR_MARK: &str = "{SEL}";
/// Where the selector goes when the template adds matchers of its own: `{SEL,…}`.
const SELECTOR_WITH_MATCHERS_MARK: &str = "{SEL,";
/// Where the range goes in a template.
const RANGE_MARK: &str = "$range";

/// A panel of the monitoring views.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Panel {
    /// Is every service up? The lowest `up`.
    Up,
    /// How many requests a second?
    Requests,
    /// Which share of requests fail? 5xx over all.
    Errors,
    /// How long do requests take? p50, p95, p99.
    Latency,
    /// Which routes are slowest? The ten slowest p95 over an hour.
    SlowRoutes,
    /// How many exports, by format and outcome?
    Exports,
    /// How many MCP tool calls, by tool and outcome?
    Mcp,
    /// How many sign-ins and other account events, by event?
    Auth,
    /// How much storage do accounts use?
    Storage,
    /// How many requests did a quota refuse?
    Quota,
    /// How much memory does each container use?
    Memory,
    /// How much CPU does each container use?
    Cpu,
}

/// Which of the environment's selectors a panel reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorKind {
    /// `LPA_METRICS_SELECTOR_<ENV>`: the services' own metrics.
    Services,
    /// `LPA_CONTAINERS_SELECTOR_<ENV>`: cAdvisor's.
    Containers,
}

/// One query of a panel: its name in the series, and its template.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PanelQuery {
    /// The series' name: the panel's, or `p50`, `p95`, `p99`.
    pub name: &'static str,
    /// The PromQL, with `{SEL}` and `$range`.
    pub template: &'static str,
}

const REQUESTS: &str = "sum(rate(http_requests_total{SEL}[5m]))";
const ERRORS: &str = "sum(rate(http_requests_total{SEL,status_class=\"5xx\"}[5m])) \
                      / sum(rate(http_requests_total{SEL}[5m]))";
const LATENCY_P50: &str = "histogram_quantile(0.5, \
                           sum by (le) (rate(http_request_duration_seconds_bucket{SEL}[5m])))";
const LATENCY_P95: &str = "histogram_quantile(0.95, \
                           sum by (le) (rate(http_request_duration_seconds_bucket{SEL}[5m])))";
const LATENCY_P99: &str = "histogram_quantile(0.99, \
                           sum by (le) (rate(http_request_duration_seconds_bucket{SEL}[5m])))";
const SLOW_ROUTES: &str = "topk(10, histogram_quantile(0.95, \
                           sum by (le, route) (rate(http_request_duration_seconds_bucket{SEL}[1h]))))";
const EXPORTS: &str = "sum by (format, outcome) (increase(exports_total{SEL}[$range]))";
const MCP: &str = "sum by (tool, outcome) (increase(mcp_tool_calls_total{SEL}[$range]))";
const AUTH: &str = "sum by (event) (increase(auth_events_total{SEL}[$range]))";
const QUOTA: &str = "sum(increase(quota_rejections_total{SEL}[$range]))";
const CPU: &str = "rate(container_cpu_usage_seconds_total{SEL}[5m])";
const LATENCY_QUERIES: [PanelQuery; 3] = [
    query("p95", LATENCY_P95),
    query("p50", LATENCY_P50),
    query("p99", LATENCY_P99),
];

const fn query(name: &'static str, template: &'static str) -> PanelQuery {
    PanelQuery { name, template }
}

impl Panel {
    /// Every panel.
    pub const ALL: [Self; 12] = [
        Self::Up,
        Self::Requests,
        Self::Errors,
        Self::Latency,
        Self::SlowRoutes,
        Self::Exports,
        Self::Mcp,
        Self::Auth,
        Self::Storage,
        Self::Quota,
        Self::Memory,
        Self::Cpu,
    ];

    /// The panels of the overview's tiles.
    pub const TILES: [Self; 7] = [
        Self::Up,
        Self::Errors,
        Self::Latency,
        Self::Requests,
        Self::Exports,
        Self::Mcp,
        Self::Storage,
    ];

    /// The panel's queries: one, or three for `latency`, p95 first — the tile's.
    #[must_use]
    pub fn queries(self) -> &'static [PanelQuery] {
        match self {
            Self::Up => &const { [query("up", "min(up{SEL})")] },
            Self::Requests => &const { [query("requests", REQUESTS)] },
            Self::Errors => &const { [query("errors", ERRORS)] },
            Self::Latency => &LATENCY_QUERIES,
            Self::SlowRoutes => &const { [query("slow_routes", SLOW_ROUTES)] },
            Self::Exports => &const { [query("exports", EXPORTS)] },
            Self::Mcp => &const { [query("mcp", MCP)] },
            Self::Auth => &const { [query("auth", AUTH)] },
            Self::Storage => &const { [query("storage", "sum(storage_used_bytes{SEL})")] },
            Self::Quota => &const { [query("quota", QUOTA)] },
            Self::Memory => &const { [query("memory", "container_memory_working_set_bytes{SEL}")] },
            Self::Cpu => &const { [query("cpu", CPU)] },
        }
    }

    /// The selector the panel's queries read.
    #[must_use]
    pub fn selector_kind(self) -> SelectorKind {
        match self {
            Self::Memory | Self::Cpu => SelectorKind::Containers,
            _ => SelectorKind::Services,
        }
    }
}

impl PanelQuery {
    /// The PromQL of the query for `selector` — label matchers without braces —, `$range` being
    /// `range`, such as `3600s`.
    #[must_use]
    pub fn promql(&self, selector: &str, range: &str) -> String {
        self.template
            .replace(SELECTOR_WITH_MATCHERS_MARK, &format!("{{{selector},"))
            .replace(SELECTOR_MARK, &format!("{{{selector}}}"))
            .replace(RANGE_MARK, range)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEL: &str = "environment=\"staging\",job=\"server\"";

    fn promql(panel: Panel) -> Vec<String> {
        panel
            .queries()
            .iter()
            .map(|query| query.promql(SEL, "3600s"))
            .collect()
    }

    #[test]
    fn every_panel_reads_its_environment_only() {
        for panel in Panel::ALL {
            for query in promql(panel) {
                assert!(
                    !query.contains("{SEL") && !query.contains("$range"),
                    "{query}"
                );
                let selections = query.matches('{').count();
                let selected = query.matches(&format!("{{{SEL}")).count();
                assert_eq!(selections, selected, "{query}");
            }
        }
    }

    #[test]
    fn the_health_traffic_and_error_panels() {
        let sel = "{environment=\"staging\",job=\"server\"}";
        assert_eq!(promql(Panel::Up), [format!("min(up{sel})")]);
        assert_eq!(
            promql(Panel::Requests),
            [format!("sum(rate(http_requests_total{sel}[5m]))")]
        );
        let errors = format!(
            "sum(rate(http_requests_total{{{SEL},status_class=\"5xx\"}}[5m])) \
             / sum(rate(http_requests_total{sel}[5m]))"
        );
        assert_eq!(promql(Panel::Errors), [errors]);
    }

    #[test]
    fn the_latency_panels() {
        let buckets = format!("rate(http_request_duration_seconds_bucket{{{SEL}}}[5m])");
        let quantile = |q: &str| format!("histogram_quantile({q}, sum by (le) ({buckets}))");
        assert_eq!(
            promql(Panel::Latency),
            [quantile("0.95"), quantile("0.5"), quantile("0.99")]
        );
        let slow = format!(
            "topk(10, histogram_quantile(0.95, sum by (le, route) \
             (rate(http_request_duration_seconds_bucket{{{SEL}}}[1h]))))"
        );
        assert_eq!(promql(Panel::SlowRoutes), [slow]);
    }

    #[test]
    fn the_counting_panels_cover_the_range() {
        let sel = format!("{{{SEL}}}");
        assert_eq!(
            promql(Panel::Exports),
            [format!(
                "sum by (format, outcome) (increase(exports_total{sel}[3600s]))"
            )]
        );
        assert_eq!(
            promql(Panel::Mcp),
            [format!(
                "sum by (tool, outcome) (increase(mcp_tool_calls_total{sel}[3600s]))"
            )]
        );
        assert_eq!(
            promql(Panel::Auth),
            [format!(
                "sum by (event) (increase(auth_events_total{sel}[3600s]))"
            )]
        );
        assert_eq!(
            promql(Panel::Storage),
            [format!("sum(storage_used_bytes{sel})")]
        );
        assert_eq!(
            promql(Panel::Quota),
            [format!("sum(increase(quota_rejections_total{sel}[3600s]))")]
        );
    }

    #[test]
    fn the_container_panels_read_the_containers_selector() {
        let sel = format!("{{{SEL}}}");
        assert_eq!(
            promql(Panel::Memory),
            [format!("container_memory_working_set_bytes{sel}")]
        );
        assert_eq!(
            promql(Panel::Cpu),
            [format!("rate(container_cpu_usage_seconds_total{sel}[5m])")]
        );
        assert_eq!(Panel::Cpu.selector_kind(), SelectorKind::Containers);
        assert_eq!(Panel::Up.selector_kind(), SelectorKind::Services);
    }
}
