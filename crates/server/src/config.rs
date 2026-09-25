//! The configuration, read from the environment and checked before anything listens: every
//! variable of docs/v1/server.md, those later tasks use included. A missing or invalid variable
//! stops the start with an error naming it; no error quotes a value, so no secret is logged.

mod callers;
mod env;
mod sections;
mod values;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use life_pixel_service::Plans;

pub use callers::{ClientVersions, TrustedProxies};
pub use env::{ConfigError, Env, FromVariable};
pub use sections::{DatabaseConfig, LegalIdentity, MailConfig, S3Config, Secrets, StorageConfig};
pub use values::{Environment, HMAC_KEY_BYTES, HmacKey, LogFilter, Origin, SecretString};

/// The development environment file, read from the working directory.
const DEVELOPMENT_ENV_FILE: &str = ".env";

/// The server's configuration: one field per variable, or per group of variables.
#[derive(Clone, Debug)]
pub struct Config {
    /// `LP_ENVIRONMENT`.
    pub environment: Environment,
    /// `LP_HTTP_ADDR`: the public listener.
    pub http_addr: SocketAddr,
    /// `LP_METRICS_ADDR`: the `/metrics` listener.
    pub metrics_addr: SocketAddr,
    /// `LP_ADMIN_API_ADDR`: the internal admin API's listener.
    pub admin_api_addr: SocketAddr,
    /// `LP_PUBLIC_URL`: the origin, for links in emails and the CSRF origin check.
    pub public_url: Origin,
    /// `LP_ALLOWED_ORIGINS`: more origins the CSRF check accepts, in development only.
    pub allowed_origins: Vec<Origin>,
    /// `LP_APP_DIR`: the built app.
    pub app_dir: PathBuf,
    /// `LP_I18N_DIR`: the catalogues.
    pub i18n_dir: PathBuf,
    /// `LP_DATABASE_URL`, `LP_DATABASE_MAX_CONNECTIONS`.
    pub database: DatabaseConfig,
    /// `LP_STORAGE_URL`, `LP_S3_*`.
    pub storage: StorageConfig,
    /// `LP_SMTP_URL`, `LP_MAIL_FROM`.
    pub mail: MailConfig,
    /// `LP_SESSION_SECRET`, `LP_EVENTS_SECRET`, `LP_EXPORT_LINK_SECRET`, `LP_ADMIN_API_SECRET`.
    pub secrets: Secrets,
    /// `LP_PLAN_FREE_STORAGE_BYTES`, `LP_PLAN_FREE_MCP_CALLS_PER_DAY`.
    pub plans: Plans,
    /// `LP_MIN_CLIENT_VERSIONS`.
    pub min_client_versions: ClientVersions,
    /// `LP_TRUSTED_PROXIES`.
    pub trusted_proxies: TrustedProxies,
    /// `LP_MCP_SERVER_NAME`: the name MCP clients see.
    pub mcp_server_name: String,
    /// `LP_LEGAL_PUBLISHER`, `LP_LEGAL_ADDRESS`, `LP_LEGAL_CONTACT`, `LP_LEGAL_DIRECTOR`,
    /// `LP_LEGAL_HOST`.
    pub legal: LegalIdentity,
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
            environment: env.parse("LP_ENVIRONMENT")?,
            http_addr: env.parse("LP_HTTP_ADDR")?,
            metrics_addr: env.parse("LP_METRICS_ADDR")?,
            admin_api_addr: env.parse("LP_ADMIN_API_ADDR")?,
            public_url: env.parse("LP_PUBLIC_URL")?,
            allowed_origins: env.parse_or_default("LP_ALLOWED_ORIGINS")?,
            app_dir: env.parse("LP_APP_DIR")?,
            i18n_dir: env.parse("LP_I18N_DIR")?,
            database: DatabaseConfig::read(&env)?,
            storage: StorageConfig::read(&env)?,
            mail: MailConfig::read(&env)?,
            secrets: Secrets::read(&env)?,
            plans: sections::read_plans(&env)?,
            min_client_versions: env.parse("LP_MIN_CLIENT_VERSIONS")?,
            trusted_proxies: env.parse_or_default("LP_TRUSTED_PROXIES")?,
            mcp_server_name: env.parse("LP_MCP_SERVER_NAME")?,
            legal: LegalIdentity::read(&env)?,
            otlp_endpoint: env.optional("OTEL_EXPORTER_OTLP_ENDPOINT"),
            log_filter: env.parse_or_default("RUST_LOG")?,
        })
    }
}

/// `LP_HTTP_ADDR` alone, for `healthcheck`, which needs nothing else.
///
/// # Errors
///
/// When `LP_HTTP_ADDR` is missing or invalid.
pub fn http_addr_from_env() -> Result<SocketAddr, ConfigError> {
    Env::new(&|variable| std::env::var(variable).ok()).parse("LP_HTTP_ADDR")
}

/// Loads `.env` from the working directory in development — when `LP_ENVIRONMENT` is unset or
/// `local` —, without overriding a variable already set. Containers get their environment from
/// Compose, and never read the file. Call it before any thread starts.
///
/// # Errors
///
/// When the file exists but cannot be read or parsed.
pub fn load_development_env() -> Result<(), dotenvy::Error> {
    let environment = std::env::var("LP_ENVIRONMENT").ok();
    let is_development = environment.is_none_or(|name| name == Environment::Local.as_str());
    let path = Path::new(DEVELOPMENT_ENV_FILE);
    if !is_development || !path.exists() {
        return Ok(());
    }
    dotenvy::from_path(path)
}
