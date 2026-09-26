//! `.env.example`'s `LPA_` block is a configuration the admin server starts with.

#![allow(clippy::unwrap_used)]

use std::collections::HashMap;

use life_pixel_admin_server::config::Config;

#[test]
fn the_example_environment_is_a_valid_configuration() {
    let example = include_str!("../../../.env.example");
    let variables: HashMap<String, String> = example
        .lines()
        .filter(|line| line.starts_with("LPA_"))
        .filter_map(|line| line.split_once('='))
        .map(|(name, value)| (name.to_owned(), value.trim_matches('"').to_owned()))
        .collect();
    let config = Config::from_lookup(&|name| variables.get(name).cloned()).unwrap();
    assert_eq!(config.http_addr.port(), 8463);
    assert_eq!(config.metrics_addr.port(), 8464);
    assert!(config.monitoring.victoria_metrics.is_none());
    assert_eq!(
        config.monitoring.environments.names(),
        ["staging", "production"]
    );
}
