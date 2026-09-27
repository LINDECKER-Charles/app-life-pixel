//! `.env.example`'s `LPA_` block is a configuration the admin server starts with locally, and
//! its public development keys — like any weak key — are refused anywhere else, by `serve` and
//! by the `create-admin` and `disable-admin` commands alike.

#![allow(clippy::unwrap_used, reason = "a panic is a failed test")]

use std::collections::HashMap;

use life_pixel_admin_server::config::{AccountsConfig, Config, ConfigError};
use sha2::{Digest, Sha256};

/// The admin server's own keys, each refused when weak outside `local`.
const KEYS: [&str; 2] = ["LPA_SESSION_SECRET", "LPA_TOTP_KEY"];
/// Keys a host refuses: zeros, and hand-made patterns of 8 and 16 distinct bytes.
const WEAK_KEYS: [&str; 3] = [
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
    "000102030405060708090a0b0c0d0e0f000102030405060708090a0b0c0d0e0f",
];
/// The least regular key a host still refuses has 16 distinct bytes: this one, 00 to 10 then
/// zeros, has 17.
const SEVENTEEN_DISTINCT_BYTES: &str =
    "000102030405060708090a0b0c0d0e0f10000000000000000000000000000000";

/// The `LPA_` variables of `.env.example`.
fn example() -> HashMap<String, String> {
    include_str!("../../../.env.example")
        .lines()
        .filter(|line| line.starts_with("LPA_"))
        .filter_map(|line| line.split_once('='))
        .map(|(name, value)| (name.to_owned(), value.trim_matches('"').to_owned()))
        .collect()
}

/// A key as random as a generated one, derived from `label`, so that none is written here.
fn strong_key(label: &str) -> String {
    Sha256::digest(label.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The example's variables on a host of `environment`, with strong keys.
fn host(environment: &str) -> HashMap<String, String> {
    let mut variables = example();
    variables.insert("LPA_ENVIRONMENT".to_owned(), environment.to_owned());
    for variable in KEYS {
        variables.insert(variable.to_owned(), strong_key(variable));
    }
    variables
}

fn read(variables: &HashMap<String, String>) -> Result<Config, ConfigError> {
    Config::from_lookup(&|name| variables.get(name).cloned())
}

fn read_accounts(variables: &HashMap<String, String>) -> Result<AccountsConfig, ConfigError> {
    AccountsConfig::from_lookup(&|name| variables.get(name).cloned())
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
    assert!(read_accounts(&example()).is_ok());
}

#[test]
fn a_host_refuses_the_development_keys_and_names_them_without_their_value() {
    for environment in ["staging", "production"] {
        let mut variables = example();
        variables.insert("LPA_ENVIRONMENT".to_owned(), environment.to_owned());
        for variable in KEYS {
            let error = read(&variables).unwrap_err();
            assert_eq!(error, ConfigError::WeakKey { variable }, "{environment}");
            assert!(!error.to_string().contains(&variables[variable]));
            variables.insert(variable.to_owned(), strong_key(variable));
        }
        assert!(read(&variables).is_ok());
    }
}

#[test]
fn a_host_refuses_any_weak_key() {
    for (variable, weak) in KEYS
        .into_iter()
        .flat_map(|key| WEAK_KEYS.map(|weak| (key, weak)))
    {
        let mut variables = host("production");
        variables.insert(variable.to_owned(), weak.to_owned());
        assert_eq!(
            read(&variables).unwrap_err(),
            ConfigError::WeakKey { variable }
        );
    }
}

#[test]
fn a_host_takes_a_key_of_17_distinct_bytes() {
    let mut variables = host("production");
    variables.insert(
        "LPA_TOTP_KEY".to_owned(),
        SEVENTEEN_DISTINCT_BYTES.to_owned(),
    );
    assert!(read(&variables).is_ok());
}

#[test]
fn the_account_commands_refuse_a_weak_totp_key_on_a_host() {
    let variable = "LPA_TOTP_KEY";
    let mut variables = example();
    variables.insert("LPA_ENVIRONMENT".to_owned(), "production".to_owned());
    assert_eq!(
        read_accounts(&variables).unwrap_err(),
        ConfigError::WeakKey { variable }
    );
    variables.insert(variable.to_owned(), strong_key(variable));
    assert!(read_accounts(&variables).is_ok());
}
