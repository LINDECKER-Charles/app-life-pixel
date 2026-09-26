//! The groups of variables: the internal admin API, the monitoring sources and their selectors,
//! the trusted proxies.

use std::collections::BTreeMap;
use std::net::IpAddr;

use ipnet::IpNet;
use reqwest::Url;

use super::env::{ConfigError, Env, FromVariable};
use super::values::SecretString;

/// An `http` or `https` URL, without a query or a fragment, its trailing slashes removed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServiceUrl(Url);

impl ServiceUrl {
    /// The URL.
    #[must_use]
    pub fn url(&self) -> &Url {
        &self.0
    }

    /// The URL of `path` under this one: `path` starts with `/`.
    #[must_use]
    pub fn join(&self, path: &str) -> String {
        format!("{}{path}", self.0.as_str().trim_end_matches('/'))
    }
}

impl FromVariable for ServiceUrl {
    const EXPECTED: &'static str = "an http or https URL without a query, such as http://vm:8428";

    fn from_variable(value: &str) -> Option<Self> {
        let url = Url::parse(value.trim_end_matches('/')).ok()?;
        let is_http = matches!(url.scheme(), "http" | "https") && url.has_host();
        let is_plain = url.query().is_none() && url.fragment().is_none();
        (is_http && is_plain).then_some(Self(url))
    }
}

/// `LPA_SERVER_ADMIN_API_URL`, `LPA_SERVER_ADMIN_API_SECRET`: where the internal admin API of the
/// server listens, and the secret it asks for.
#[derive(Clone, Debug)]
pub struct ServerAdminApi {
    /// The API's root, such as `http://127.0.0.1:8462/internal/admin/v1`.
    pub url: ServiceUrl,
    /// The server's `LP_ADMIN_API_SECRET`.
    pub secret: SecretString,
}

impl ServerAdminApi {
    /// The two variables.
    ///
    /// # Errors
    ///
    /// When one is missing or invalid.
    pub fn read(env: &Env<'_>) -> Result<Self, ConfigError> {
        Ok(Self {
            url: env.parse("LPA_SERVER_ADMIN_API_URL")?,
            secret: env.parse("LPA_SERVER_ADMIN_API_SECRET")?,
        })
    }
}

/// `LPA_ENVIRONMENTS`: the environments the monitoring shows, lowercase names separated by
/// commas.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentNames(Vec<String>);

impl EnvironmentNames {
    /// The names, in the variable's order.
    #[must_use]
    pub fn names(&self) -> &[String] {
        &self.0
    }
}

impl FromVariable for EnvironmentNames {
    const EXPECTED: &'static str =
        "lowercase names separated by commas, such as staging,production";

    fn from_variable(value: &str) -> Option<Self> {
        let is_name = |name: &str| {
            !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        };
        value
            .split(',')
            .map(|name| Some(name.trim().to_owned()).filter(|name| is_name(name)))
            .collect::<Option<_>>()
            .map(Self)
    }
}

/// The label selectors infra-vps gives one environment, each without its braces; `None` when
/// unset.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Selectors {
    /// `LPA_METRICS_SELECTOR_<ENV>`: the services' metrics, such as `environment="staging"`.
    pub metrics: Option<String>,
    /// `LPA_CONTAINERS_SELECTOR_<ENV>`: the containers' metrics of cAdvisor.
    pub containers: Option<String>,
    /// `LPA_LOGS_SELECTOR_<ENV>`: the stream filter of the environment's logs.
    pub logs: Option<String>,
}

/// The monitoring sources: VictoriaMetrics through its query-only proxy, VictoriaLogs,
/// Alertmanager and Grafana. A source left empty is not configured.
#[derive(Clone, Debug, Default)]
pub struct MonitoringConfig {
    /// `LPA_ENVIRONMENTS`.
    pub environments: EnvironmentNames,
    /// `LPA_VICTORIAMETRICS_URL`.
    pub victoria_metrics: Option<ServiceUrl>,
    /// `LPA_VICTORIALOGS_URL`.
    pub victoria_logs: Option<ServiceUrl>,
    /// `LPA_ALERTMANAGER_URL`.
    pub alertmanager: Option<ServiceUrl>,
    /// `LPA_GRAFANA_URL`.
    pub grafana: Option<ServiceUrl>,
    /// `LPA_GRAFANA_LOGS_DATASOURCE`, `LPA_GRAFANA_TRACES_DATASOURCE`: the uids of Grafana's
    /// VictoriaLogs and traces data sources.
    pub grafana_datasources: GrafanaDatasources,
    /// `LPA_ALERTS_FILTER`: Alertmanager matchers separated by commas; `{ENV}` stands for the
    /// environment.
    pub alerts_filter: Option<String>,
    /// The selectors of each environment of `environments`.
    pub selectors: BTreeMap<String, Selectors>,
}

/// The uids of Grafana's data sources the links open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrafanaDatasources {
    /// The VictoriaLogs data source.
    pub logs: String,
    /// The traces data source.
    pub traces: String,
}

impl Default for GrafanaDatasources {
    fn default() -> Self {
        Self {
            logs: "victorialogs".to_owned(),
            traces: "victoriatraces".to_owned(),
        }
    }
}

impl MonitoringConfig {
    /// The monitoring variables, the selectors of each environment included.
    ///
    /// # Errors
    ///
    /// When a URL or `LPA_ENVIRONMENTS` is invalid.
    pub fn read(env: &Env<'_>) -> Result<Self, ConfigError> {
        let environments: EnvironmentNames = env.parse_or_default("LPA_ENVIRONMENTS")?;
        let selectors = environments
            .names()
            .iter()
            .map(|name| (name.clone(), read_selectors(env, name)))
            .collect();
        let defaults = GrafanaDatasources::default();
        Ok(Self {
            environments,
            victoria_metrics: optional_url(env, "LPA_VICTORIAMETRICS_URL")?,
            victoria_logs: optional_url(env, "LPA_VICTORIALOGS_URL")?,
            alertmanager: optional_url(env, "LPA_ALERTMANAGER_URL")?,
            grafana: optional_url(env, "LPA_GRAFANA_URL")?,
            grafana_datasources: GrafanaDatasources {
                logs: env
                    .optional("LPA_GRAFANA_LOGS_DATASOURCE")
                    .unwrap_or(defaults.logs),
                traces: env
                    .optional("LPA_GRAFANA_TRACES_DATASOURCE")
                    .unwrap_or(defaults.traces),
            },
            alerts_filter: env.optional("LPA_ALERTS_FILTER"),
            selectors,
        })
    }
}

fn optional_url(env: &Env<'_>, variable: &'static str) -> Result<Option<ServiceUrl>, ConfigError> {
    let Some(value) = env.optional(variable) else {
        return Ok(None);
    };
    ServiceUrl::from_variable(&value)
        .map(Some)
        .ok_or(ConfigError::Invalid {
            variable,
            expected: ServiceUrl::EXPECTED,
        })
}

/// The selectors of the environment `name`, their enclosing braces removed.
fn read_selectors(env: &Env<'_>, name: &str) -> Selectors {
    let suffix = name.to_ascii_uppercase();
    let read = |prefix: &str| {
        env.optional(&format!("{prefix}{suffix}"))
            .map(|selector| strip_braces(&selector).to_owned())
            .filter(|selector| !selector.is_empty())
    };
    Selectors {
        metrics: read("LPA_METRICS_SELECTOR_"),
        containers: read("LPA_CONTAINERS_SELECTOR_"),
        logs: read("LPA_LOGS_SELECTOR_"),
    }
}

fn strip_braces(selector: &str) -> &str {
    let trimmed = selector.trim();
    trimmed
        .strip_prefix('{')
        .and_then(|inner| inner.strip_suffix('}'))
        .unwrap_or(trimmed)
        .trim()
}

/// `LPA_TRUSTED_PROXIES`: the networks of the proxies in front of the admin server, whose
/// `X-Forwarded-For` is believed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TrustedProxies(Vec<IpNet>);

impl TrustedProxies {
    /// Whether `address` belongs to a trusted proxy.
    #[must_use]
    pub fn contains(&self, address: IpAddr) -> bool {
        self.0.iter().any(|network| network.contains(&address))
    }
}

impl FromVariable for TrustedProxies {
    const EXPECTED: &'static str = "networks separated by commas, such as 172.16.0.0/12,10.0.0.1";

    fn from_variable(value: &str) -> Option<Self> {
        value
            .split(',')
            .map(|network| {
                let network = network.trim();
                network
                    .parse()
                    .ok()
                    .or_else(|| network.parse::<IpAddr>().ok().map(IpNet::from))
            })
            .collect::<Option<_>>()
            .map(Self)
    }
}
