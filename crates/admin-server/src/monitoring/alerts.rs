//! Alertmanager: the firing alerts of `/api/v2/alerts`, active and neither silenced nor
//! inhibited, filtered by `LPA_ALERTS_FILTER` — Alertmanager matchers separated by commas, where
//! `{ENV}` stands for the environment. The VPS's Alertmanager serves other projects too, so an
//! empty filter is a source not configured.

use std::collections::BTreeMap;

use serde::Deserialize;

use super::schema::Alert;
use super::source::{Endpoint, Source, SourceError};
use crate::config::ServiceUrl;

const ALERTS_PATH: &str = "/api/v2/alerts";
/// Where the environment's name goes in `LPA_ALERTS_FILTER`.
const ENVIRONMENT_MARK: &str = "{ENV}";

/// An alert, as Alertmanager's API v2 gives it.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GettableAlert {
    fingerprint: String,
    #[serde(default)]
    labels: BTreeMap<String, String>,
    #[serde(default)]
    annotations: BTreeMap<String, String>,
    starts_at: String,
    status: AlertStatus,
}

#[derive(Deserialize)]
struct AlertStatus {
    state: String,
}

/// The matchers of `filter` for `environment`: split at the commas outside quotes, `{ENV}`
/// replaced.
#[must_use]
pub fn matchers(filter: &str, environment: &str) -> Vec<String> {
    let mut matchers = Vec::new();
    let mut current = String::new();
    let mut is_quoted = false;
    let mut is_escaped = false;
    for character in filter.chars() {
        if character == ',' && !is_quoted {
            matchers.push(std::mem::take(&mut current));
            continue;
        }
        is_quoted ^= character == '"' && !is_escaped;
        is_escaped = character == '\\' && !is_escaped;
        current.push(character);
    }
    matchers.push(current);
    matchers
        .iter()
        .map(|matcher| matcher.trim().replace(ENVIRONMENT_MARK, environment))
        .filter(|matcher| !matcher.is_empty())
        .collect()
}

/// An Alertmanager to query.
#[derive(Clone, Debug)]
pub struct AlertsSource<'a> {
    /// The client.
    pub client: &'a reqwest::Client,
    /// `LPA_ALERTMANAGER_URL`.
    pub url: &'a ServiceUrl,
}

impl AlertsSource<'_> {
    /// The firing alerts `matchers` select, newest first.
    ///
    /// # Errors
    ///
    /// [`SourceError::Unavailable`].
    pub async fn firing(&self, matchers: &[String]) -> Result<Vec<Alert>, SourceError> {
        let mut query = vec![
            ("active", "true".to_owned()),
            ("silenced", "false".to_owned()),
            ("inhibited", "false".to_owned()),
        ];
        query.extend(matchers.iter().map(|matcher| ("filter", matcher.clone())));
        let endpoint = Endpoint {
            client: self.client,
            source: Source::Alertmanager,
            url: self.url.join(ALERTS_PATH),
        };
        let found: Vec<GettableAlert> = endpoint.get_json(&query).await?;
        let mut alerts: Vec<Alert> = found.into_iter().map(alert).collect();
        alerts.sort_by(|a, b| b.starts_at.cmp(&a.starts_at));
        Ok(alerts)
    }
}

fn alert(found: GettableAlert) -> Alert {
    let GettableAlert {
        fingerprint,
        labels,
        mut annotations,
        starts_at,
        status,
    } = found;
    Alert {
        fingerprint,
        name: labels.get("alertname").cloned().unwrap_or_default(),
        severity: labels.get("severity").cloned(),
        state: status.state,
        starts_at,
        summary: annotations.remove("summary"),
        description: annotations.remove("description"),
        labels,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_filter_splits_outside_quotes_and_names_the_environment() {
        let filter = r#"project="life-pixel", environment="{ENV}", team=~"a,b""#;
        assert_eq!(
            matchers(filter, "staging"),
            [
                r#"project="life-pixel""#,
                r#"environment="staging""#,
                r#"team=~"a,b""#
            ]
        );
        assert_eq!(matchers(r#"x="a\",b""#, "p"), [r#"x="a\",b""#]);
        assert!(matchers(" , ", "p").is_empty());
    }
}
