//! The configuration, read from the `LPA_` variables of docs/v1/support-admin.md and checked
//! before anything listens. A missing or invalid variable stops the start with an error naming
//! it; no error quotes a value, so no secret is logged.

mod env;
mod sections;
mod values;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

pub use env::{ConfigError, Env, FromVariable};
pub use sections::{
    EnvironmentNames, GrafanaDatasources, MonitoringConfig, Selectors, ServerAdminApi, ServiceUrl,
    TrustedProxies,
};
pub use values::{Environment, KEY_BYTES, Key, LogFilter, Origin, SecretString};

/// The development environment file, read from the working directory.
const DEVELOPMENT_ENV_FILE: &str = ".env";

/// The admin server's configuration: one field per variable, or per group of variables.
#[derive(Clone, Debug)]
pub struct Config {
    /// `LPA_ENVIRONMENT`: the environment this console administers.
    pub environment: Environment,
    /// `LPA_HTTP_ADDR`: the console and its API.
    pub http_addr: SocketAddr,
    /// `LPA_METRICS_ADDR`: the `/metrics` listener.
    pub metrics_addr: SocketAddr,
    /// `LPA_PUBLIC_URL`: the console's origin, for the CSRF origin check.
    pub public_url: Origin,
    /// `LPA_ALLOWED_ORIGINS`: more origins the CSRF check accepts, for `ng serve`.
    pub allowed_origins: Vec<Origin>,
    /// `LPA_APP_DIR`: the built console.
    pub app_dir: PathBuf,
    /// `LPA_I18N_DIR`: the catalogues the console reads.
    pub i18n_dir: PathBuf,
    /// `LPA_DATABASE_URL`: the admin server's own database.
    pub database_url: SecretString,
    /// `LPA_SESSION_SECRET`: the key of the CSRF tokens.
    pub session_secret: Key,
    /// `LPA_TOTP_KEY`: the key encrypting the admins' TOTP secrets.
    pub totp_key: Key,
    /// `LPA_SERVER_ADMIN_API_URL`, `LPA_SERVER_ADMIN_API_SECRET`.
    pub server_admin_api: ServerAdminApi,
    /// `LPA_ENVIRONMENTS`, the sources, the selectors and `LPA_ALERTS_FILTER`.
    pub monitoring: MonitoringConfig,
    /// `LPA_TRUSTED_PROXIES`: the edge, whose `X-Forwarded-For` names the client.
    pub trusted_proxies: TrustedProxies,
    /// `OTEL_EXPORTER_OTLP_ENDPOINT`: traces are sent only when it is set.
    pub otlp_endpoint: Option<String>,
    /// `RUST_LOG`, `info` when unset.
    pub log_filter: LogFilter,
}

impl Config {
    /// The configuration of the process environment.
    ///
    /// # Errors
    ///
    /// The first variable that is missing or invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(&|variable| std::env::var(variable).ok())
    }

    /// The configuration of the variables `lookup` finds.
    ///
    /// # Errors
    ///
    /// The first variable that is missing or invalid.
    pub fn from_lookup(lookup: &dyn Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let env = Env::new(lookup);
        Ok(Self {
            environment: env.parse("LPA_ENVIRONMENT")?,
            http_addr: env.parse("LPA_HTTP_ADDR")?,
            metrics_addr: env.parse("LPA_METRICS_ADDR")?,
            public_url: env.parse("LPA_PUBLIC_URL")?,
            allowed_origins: env.parse_or_default("LPA_ALLOWED_ORIGINS")?,
            app_dir: env.parse("LPA_APP_DIR")?,
            i18n_dir: env.parse("LPA_I18N_DIR")?,
            database_url: env.parse("LPA_DATABASE_URL")?,
            session_secret: env.parse("LPA_SESSION_SECRET")?,
            totp_key: env.parse("LPA_TOTP_KEY")?,
            server_admin_api: ServerAdminApi::read(&env)?,
            monitoring: MonitoringConfig::read(&env)?,
            trusted_proxies: env.parse_or_default("LPA_TRUSTED_PROXIES")?,
            otlp_endpoint: env.optional("OTEL_EXPORTER_OTLP_ENDPOINT"),
            log_filter: env.parse_or_default("RUST_LOG")?,
        })
    }
}

/// What `create-admin` and `disable-admin` need, and nothing else: `LPA_DATABASE_URL` and
/// `LPA_TOTP_KEY`.
#[derive(Clone, Debug)]
pub struct AccountsConfig {
    /// `LPA_DATABASE_URL`.
    pub database_url: SecretString,
    /// `LPA_TOTP_KEY`.
    pub totp_key: Key,
}

impl AccountsConfig {
    /// The two variables of the process environment.
    ///
    /// # Errors
    ///
    /// When one is missing or invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        let env = Env::new(&|variable| std::env::var(variable).ok());
        Ok(Self {
            database_url: env.parse("LPA_DATABASE_URL")?,
            totp_key: env.parse("LPA_TOTP_KEY")?,
        })
    }
}

/// `LPA_HTTP_ADDR` alone, for `healthcheck`, which needs nothing else.
///
/// # Errors
///
/// When `LPA_HTTP_ADDR` is missing or invalid.
pub fn http_addr_from_env() -> Result<SocketAddr, ConfigError> {
    Env::new(&|variable| std::env::var(variable).ok()).parse("LPA_HTTP_ADDR")
}

/// Loads `.env` from the working directory in development — when `LPA_ENVIRONMENT` is unset or
/// `local` —, without overriding a variable already set. Containers get their environment from
/// Compose, and never read the file. Call it before any thread starts.
///
/// # Errors
///
/// When the file exists but cannot be read or parsed.
pub fn load_development_env() -> Result<(), dotenvy::Error> {
    let environment = std::env::var("LPA_ENVIRONMENT").ok();
    let is_development = environment.is_none_or(|name| name == Environment::Local.as_str());
    let path = Path::new(DEVELOPMENT_ENV_FILE);
    if !is_development || !path.exists() {
        return Ok(());
    }
    dotenvy::from_path(path)
}
