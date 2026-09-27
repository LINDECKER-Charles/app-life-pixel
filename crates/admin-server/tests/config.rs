//! `.env.example`'s `LPA_` block is a configuration the admin server starts with locally, and
//! its public development keys are refused anywhere else.

#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashMap;

use common::{SECRET, TOTP_KEY};
use life_pixel_admin_server::config::{Config, ConfigError};

/// The `LPA_` variables of `.env.example`.
fn example() -> HashMap<String, String> {
    include_str!("../../../.env.example")
        .lines()
        .filter(|line| line.starts_with("LPA_"))
        .filter_map(|line| line.split_once('='))
        .map(|(name, value)| (name.to_owned(), value.trim_matches('"').to_owned()))
        .collect()
}

fn read(variables: &HashMap<String, String>) -> Result<Config, ConfigError> {
    Config::from_lookup(&|name| variables.get(name).cloned())
}

#[test]
fn the_example_environment_is_a_valid_configuration() {
    let config = read(&example()).unwrap();
    assert_eq!(config.http_addr.port(), 8463);
    assert_eq!(config.metrics_addr.port(), 8464);
    assert!(config.monitoring.victoria_metrics.is_none());
    assert_eq!(
        config.monitoring.environments.names(),
        ["staging", "production"]
    );
}

#[test]
fn a_host_refuses_the_development_keys_and_names_them_without_their_value() {
    for environment in ["staging", "production"] {
        let mut variables = example();
        variables.insert("LPA_ENVIRONMENT".to_owned(), environment.to_owned());
        let error = read(&variables).unwrap_err();
        let variable = "LPA_SESSION_SECRET";
        assert_eq!(error, ConfigError::WeakKey { variable }, "{environment}");
        assert!(!error.to_string().contains(&variables[variable]));
        variables.insert(variable.to_owned(), SECRET.to_owned());
        let variable = "LPA_TOTP_KEY";
        assert_eq!(
            read(&variables).unwrap_err(),
            ConfigError::WeakKey { variable }
        );
    }
}

#[test]
fn a_host_takes_random_keys() {
    let mut variables = example();
    variables.insert("LPA_ENVIRONMENT".to_owned(), "production".to_owned());
    variables.insert("LPA_SESSION_SECRET".to_owned(), SECRET.to_owned());
    variables.insert("LPA_TOTP_KEY".to_owned(), TOTP_KEY.to_owned());
    assert!(read(&variables).is_ok());
}
