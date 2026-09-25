//! The configuration: every variable read, and a missing or invalid one named, never quoted.

mod common;

use std::collections::HashMap;
use std::path::Path;

use common::{SECRET, local_env, read_config};
use life_pixel_server::config::{ConfigError, Environment, StorageConfig};

/// The variables a configuration cannot do without.
const REQUIRED: [&str; 30] = [
    "LP_ENVIRONMENT",
    "LP_HTTP_ADDR",
    "LP_METRICS_ADDR",
    "LP_ADMIN_API_ADDR",
    "LP_PUBLIC_URL",
    "LP_APP_DIR",
    "LP_I18N_DIR",
    "LP_DATABASE_URL",
    "LP_DATABASE_MAX_CONNECTIONS",
    "LP_STORAGE_URL",
    "LP_S3_ENDPOINT",
    "LP_S3_REGION",
    "LP_S3_ACCESS_KEY_ID",
    "LP_S3_SECRET_ACCESS_KEY",
    "LP_S3_PATH_STYLE",
    "LP_SMTP_URL",
    "LP_MAIL_FROM",
    "LP_SESSION_SECRET",
    "LP_EVENTS_SECRET",
    "LP_EXPORT_LINK_SECRET",
    "LP_ADMIN_API_SECRET",
    "LP_PLAN_FREE_STORAGE_BYTES",
    "LP_PLAN_FREE_MCP_CALLS_PER_DAY",
    "LP_MIN_CLIENT_VERSIONS",
    "LP_MCP_SERVER_NAME",
    "LP_LEGAL_PUBLISHER",
    "LP_LEGAL_ADDRESS",
    "LP_LEGAL_CONTACT",
    "LP_LEGAL_DIRECTOR",
    "LP_LEGAL_HOST",
];

/// Invalid values, each with the variable it is given to.
const INVALID: [(&str, &str); 14] = [
    ("LP_ENVIRONMENT", "dev"),
    ("LP_HTTP_ADDR", "localhost"),
    ("LP_PUBLIC_URL", "http://localhost:8460/app"),
    ("LP_ALLOWED_ORIGINS", "http://localhost:4260,ftp://files"),
    ("LP_DATABASE_URL", "mysql://root:hunter2@db/life_pixel"),
    ("LP_DATABASE_MAX_CONNECTIONS", "0"),
    ("LP_STORAGE_URL", "gs://bucket"),
    ("LP_S3_PATH_STYLE", "yes"),
    ("LP_SMTP_URL", "http://mail"),
    ("LP_SESSION_SECRET", "hunter2hunter2"),
    ("LP_PLAN_FREE_STORAGE_BYTES", "-1"),
    ("LP_MIN_CLIENT_VERSIONS", "web=latest"),
    ("LP_TRUSTED_PROXIES", "10.0.0.0/33"),
    ("RUST_LOG", "info,[unclosed"),
];

fn local() -> HashMap<String, String> {
    local_env(Path::new("frontend/dist/app/browser"))
}

#[test]
fn the_local_values_make_a_configuration() {
    let config = read_config(&local()).unwrap();
    assert_eq!(config.environment, Environment::Local);
    assert_eq!(config.http_addr, "127.0.0.1:8460".parse().unwrap());
    assert_eq!(config.public_url.as_str(), "http://localhost:8460");
    assert_eq!(config.allowed_origins.len(), 1);
    assert_eq!(config.plans.free_storage_bytes, 100_000_000);
    assert!(config.min_client_versions.minimum("desktop").is_some());
    assert!(config.otlp_endpoint.is_none());
    assert_eq!(config.log_filter.as_str(), "info");
}

#[test]
fn the_database_and_the_store_come_from_their_urls() {
    let config = read_config(&local()).unwrap();
    assert_eq!(config.database.address, "127.0.0.1:5460");
    assert_eq!(config.database.max_connections.get(), 10);
    let StorageConfig::S3 { bucket, settings } = config.storage else {
        panic!("not an S3 store");
    };
    assert_eq!(bucket, "life-pixel-local");
    assert!(settings.path_style);
}

#[test]
fn the_example_environment_makes_a_configuration() {
    let path = format!("{}/../../.env.example", env!("CARGO_MANIFEST_DIR"));
    let example: HashMap<String, String> = dotenvy::from_path_iter(&path)
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    let config = read_config(&example).unwrap();
    assert_eq!(config.environment, Environment::Local);
    assert_eq!(config.mail.from, "Life Pixel <no-reply@localhost>");
    assert_eq!(config.legal.publisher, "Life Pixel (local)");
    assert!(config.otlp_endpoint.is_none());
}

#[test]
fn a_missing_variable_is_named() {
    for variable in REQUIRED {
        let mut env = local();
        env.remove(variable);
        let error = read_config(&env).unwrap_err();
        assert_eq!(error, ConfigError::Missing { variable });
        assert_eq!(error.to_string(), format!("{variable} is missing"));
    }
}

#[test]
fn an_invalid_variable_is_named_without_its_value() {
    for (variable, value) in INVALID {
        let mut env = local();
        env.insert(variable.to_owned(), value.to_owned());
        let error = read_config(&env).unwrap_err();
        assert_eq!(error.variable(), variable, "{value}");
        let message = error.to_string();
        assert!(message.starts_with(&format!("{variable} is invalid: expected ")));
        assert!(!message.contains(value), "{message}");
    }
}

#[test]
fn the_optional_variables_have_defaults() {
    let mut env = local();
    for variable in ["LP_ALLOWED_ORIGINS", "LP_TRUSTED_PROXIES", "RUST_LOG"] {
        env.remove(variable);
    }
    env.insert(
        "OTEL_EXPORTER_OTLP_ENDPOINT".to_owned(),
        "http://otel:4318".to_owned(),
    );
    let config = read_config(&env).unwrap();
    assert!(config.allowed_origins.is_empty());
    assert!(!config.trusted_proxies.contains([10, 0, 0, 1].into()));
    assert_eq!(config.log_filter.as_str(), "info");
    assert_eq!(config.otlp_endpoint.as_deref(), Some("http://otel:4318"));
}

#[test]
fn a_file_store_needs_no_s3_settings() {
    let mut env = local();
    env.retain(|name, _| !name.starts_with("LP_S3_"));
    env.insert(
        "LP_STORAGE_URL".to_owned(),
        "file:///var/lib/life-pixel".to_owned(),
    );
    let config = read_config(&env).unwrap();
    assert!(
        matches!(config.storage, StorageConfig::File { ref root } if root.ends_with("life-pixel"))
    );
}

#[test]
fn secrets_never_show_in_the_debug_output() {
    let mut env = local();
    let password = "s3cr3t-password";
    env.insert(
        "LP_DATABASE_URL".to_owned(),
        format!("postgres://lp:{password}@db:5432/lp"),
    );
    env.insert("LP_S3_SECRET_ACCESS_KEY".to_owned(), password.to_owned());
    let debug = format!("{:?}", read_config(&env).unwrap());
    assert!(!debug.contains(password));
    assert!(!debug.contains(SECRET));
    assert!(!debug.contains(&SECRET[..16]));
}
