//! The groups of variables: database, storage, mail, secrets, plans, legal identity. Each reads
//! its variables one line each.

use std::num::NonZeroU32;
use std::path::PathBuf;

use life_pixel_service::Plans;

use super::env::{ConfigError, Env, FromVariable};
use super::values::{HmacKey, SecretString};

const POSTGRES_SCHEMES: [&str; 2] = ["postgres://", "postgresql://"];
const POSTGRES_DEFAULT_PORT: &str = "5432";

/// `LP_DATABASE_URL` and `LP_DATABASE_MAX_CONNECTIONS`.
#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    /// The Postgres URL, credentials included: never logged.
    pub url: SecretString,
    /// The `host:port` of the URL, which the readiness probe connects to.
    pub address: String,
    /// The size of the connection pool.
    pub max_connections: NonZeroU32,
}

impl DatabaseConfig {
    pub(super) fn read(env: &Env<'_>) -> Result<Self, ConfigError> {
        let DatabaseUrl { url, address } = env.parse("LP_DATABASE_URL")?;
        Ok(Self {
            url,
            address,
            max_connections: env.parse("LP_DATABASE_MAX_CONNECTIONS")?,
        })
    }
}

struct DatabaseUrl {
    url: SecretString,
    address: String,
}

impl FromVariable for DatabaseUrl {
    const EXPECTED: &'static str = "a URL such as postgres://user:password@host:5432/database";

    fn from_variable(value: &str) -> Option<Self> {
        let rest = POSTGRES_SCHEMES
            .iter()
            .find_map(|scheme| value.strip_prefix(scheme))?;
        let authority = rest.split(['/', '?']).next().unwrap_or_default();
        let host = authority.rsplit('@').next().unwrap_or_default();
        let has_port = host
            .rsplit_once(':')
            .is_some_and(|(name, port)| !name.ends_with(':') && port.parse::<u16>().is_ok());
        let address = match (host.is_empty(), has_port) {
            (true, _) => return None,
            (false, true) => host.to_owned(),
            (false, false) => format!("{host}:{POSTGRES_DEFAULT_PORT}"),
        };
        Some(Self {
            url: SecretString::from(value.to_owned()),
            address,
        })
    }
}

/// `LP_STORAGE_URL`, with the `LP_S3_*` settings when it names a bucket.
#[derive(Clone, Debug)]
pub enum StorageConfig {
    /// `s3://<bucket>`.
    S3 {
        /// The bucket's name.
        bucket: String,
        /// The S3 settings.
        settings: S3Config,
    },
    /// `file:///<folder>`, the self-hosting default.
    File {
        /// The folder the objects are stored in.
        root: PathBuf,
    },
}

impl StorageConfig {
    pub(super) fn read(env: &Env<'_>) -> Result<Self, ConfigError> {
        Ok(match env.parse("LP_STORAGE_URL")? {
            StorageUrl::S3(bucket) => Self::S3 {
                bucket,
                settings: S3Config::read(env)?,
            },
            StorageUrl::File(root) => Self::File { root },
        })
    }
}

enum StorageUrl {
    S3(String),
    File(PathBuf),
}

impl FromVariable for StorageUrl {
    const EXPECTED: &'static str = "s3://<bucket> or file:///<folder>";

    fn from_variable(value: &str) -> Option<Self> {
        if let Some(bucket) = value.strip_prefix("s3://") {
            let is_bucket = !bucket.is_empty() && !bucket.contains('/');
            return is_bucket.then(|| Self::S3(bucket.to_owned()));
        }
        let root = value.strip_prefix("file://")?;
        root.starts_with('/')
            .then(|| Self::File(PathBuf::from(root)))
    }
}

/// The `LP_S3_*` settings.
#[derive(Clone, Debug)]
pub struct S3Config {
    /// `LP_S3_ENDPOINT`.
    pub endpoint: String,
    /// `LP_S3_REGION`.
    pub region: String,
    /// `LP_S3_ACCESS_KEY_ID`.
    pub access_key_id: SecretString,
    /// `LP_S3_SECRET_ACCESS_KEY`.
    pub secret_access_key: SecretString,
    /// `LP_S3_PATH_STYLE`: path-style URLs, as S3Mock needs.
    pub path_style: bool,
}

impl S3Config {
    fn read(env: &Env<'_>) -> Result<Self, ConfigError> {
        Ok(Self {
            endpoint: env.parse("LP_S3_ENDPOINT")?,
            region: env.parse("LP_S3_REGION")?,
            access_key_id: env.parse("LP_S3_ACCESS_KEY_ID")?,
            secret_access_key: env.parse("LP_S3_SECRET_ACCESS_KEY")?,
            path_style: env.parse("LP_S3_PATH_STYLE")?,
        })
    }
}

/// `LP_SMTP_URL` and `LP_MAIL_FROM`.
#[derive(Clone, Debug)]
pub struct MailConfig {
    /// The SMTP server in URL form, credentials included: never logged.
    pub smtp_url: SecretString,
    /// The sender, such as `Life Pixel <no-reply@lifepixel.tech>`.
    pub from: String,
}

impl MailConfig {
    pub(super) fn read(env: &Env<'_>) -> Result<Self, ConfigError> {
        Ok(Self {
            smtp_url: env.parse::<SmtpUrl>("LP_SMTP_URL")?.0,
            from: env.parse("LP_MAIL_FROM")?,
        })
    }
}

struct SmtpUrl(SecretString);

impl FromVariable for SmtpUrl {
    const EXPECTED: &'static str = "a URL such as smtp://host:587 or smtps://user:password@host";

    fn from_variable(value: &str) -> Option<Self> {
        let is_smtp = ["smtp://", "smtps://"]
            .iter()
            .any(|scheme| value.len() > scheme.len() && value.starts_with(scheme));
        is_smtp.then(|| Self(SecretString::from(value.to_owned())))
    }
}

/// The HMAC keys.
#[derive(Clone, Debug)]
pub struct Secrets {
    /// `LP_SESSION_SECRET`: CSRF tokens.
    pub session: HmacKey,
    /// `LP_EVENTS_SECRET`: the subjects of product events.
    pub events: HmacKey,
    /// `LP_EXPORT_LINK_SECRET`: signed export links.
    pub export_link: HmacKey,
    /// `LP_ADMIN_API_SECRET`: the internal admin API, shared with the admin server.
    pub admin_api: HmacKey,
}

impl Secrets {
    pub(super) fn read(env: &Env<'_>) -> Result<Self, ConfigError> {
        Ok(Self {
            session: env.parse("LP_SESSION_SECRET")?,
            events: env.parse("LP_EVENTS_SECRET")?,
            export_link: env.parse("LP_EXPORT_LINK_SECRET")?,
            admin_api: env.parse("LP_ADMIN_API_SECRET")?,
        })
    }
}

/// `LP_PLAN_FREE_*`: the free plan.
pub(super) fn read_plans(env: &Env<'_>) -> Result<Plans, ConfigError> {
    Ok(Plans {
        free_storage_bytes: env.parse("LP_PLAN_FREE_STORAGE_BYTES")?,
        free_mcp_calls_per_day: env.parse("LP_PLAN_FREE_MCP_CALLS_PER_DAY")?,
    })
}

/// `LP_LEGAL_*`: who publishes this server, for the legal pages.
#[derive(Clone, Debug)]
pub struct LegalIdentity {
    /// `LP_LEGAL_PUBLISHER`.
    pub publisher: String,
    /// `LP_LEGAL_ADDRESS`.
    pub address: String,
    /// `LP_LEGAL_CONTACT`.
    pub contact: String,
    /// `LP_LEGAL_DIRECTOR`.
    pub director: String,
    /// `LP_LEGAL_HOST`.
    pub host: String,
}

impl LegalIdentity {
    pub(super) fn read(env: &Env<'_>) -> Result<Self, ConfigError> {
        Ok(Self {
            publisher: env.parse("LP_LEGAL_PUBLISHER")?,
            address: env.parse("LP_LEGAL_ADDRESS")?,
            contact: env.parse("LP_LEGAL_CONTACT")?,
            director: env.parse("LP_LEGAL_DIRECTOR")?,
            host: env.parse("LP_LEGAL_HOST")?,
        })
    }
}
