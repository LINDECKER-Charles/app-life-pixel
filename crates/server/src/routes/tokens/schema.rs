//! The token routes' bodies: tokens as the routes answer them — never their hash, their secret
//! only once —, and what creating one reads.

use life_pixel_service::tokens::ports::AccessTokenRecord;
use life_pixel_service::tokens::{CreatedToken, NewToken, TokenScope};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::ToSchema;

/// What a token grants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccessTokenScope {
    /// Reading animations, their previews and their snippets.
    Read,
    /// Creating and changing animations.
    Write,
    /// Exporting animations as files.
    Export,
}

impl From<TokenScope> for AccessTokenScope {
    fn from(scope: TokenScope) -> Self {
        match scope {
            TokenScope::Read => Self::Read,
            TokenScope::Write => Self::Write,
            TokenScope::Export => Self::Export,
        }
    }
}

impl From<AccessTokenScope> for TokenScope {
    fn from(scope: AccessTokenScope) -> Self {
        match scope {
            AccessTokenScope::Read => Self::Read,
            AccessTokenScope::Write => Self::Write,
            AccessTokenScope::Export => Self::Export,
        }
    }
}

/// A token of the account, without its secret.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccessTokenSummary {
    /// Its id.
    #[schema(format = "uuid")]
    pub id: String,
    /// The name its owner gave it.
    pub name: String,
    /// The start of its secret, `lp_pat_` and 4 characters, to recognize it.
    pub prefix: String,
    /// What it grants.
    pub scopes: Vec<AccessTokenScope>,
    /// When it was created.
    #[schema(format = DateTime)]
    pub created_at: String,
    /// When it stops working.
    #[schema(format = DateTime)]
    pub expires_at: String,
    /// When it was last used, to the minute; never used when absent.
    #[schema(format = DateTime)]
    pub last_used_at: Option<String>,
}

impl From<AccessTokenRecord> for AccessTokenSummary {
    fn from(record: AccessTokenRecord) -> Self {
        Self {
            id: record.id.to_string(),
            name: record.name,
            prefix: record.prefix,
            scopes: record.scopes.into_iter().map(Into::into).collect(),
            created_at: timestamp(record.created_at),
            expires_at: timestamp(record.expires_at),
            last_used_at: record.last_used_at.map(timestamp),
        }
    }
}

/// The MCP server a token is for: what `claude mcp add` registers.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    /// The name to register it under: `LP_MCP_SERVER_NAME`, distinct per environment.
    pub server_name: String,
    /// The endpoint's URL.
    pub url: String,
}

/// A token just created: its secret is in this answer only.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatedAccessToken {
    /// Its id.
    #[schema(format = "uuid")]
    pub id: String,
    /// The name its owner gave it.
    pub name: String,
    /// The start of its secret, `lp_pat_` and 4 characters, to recognize it.
    pub prefix: String,
    /// What it grants.
    pub scopes: Vec<AccessTokenScope>,
    /// When it was created.
    #[schema(format = DateTime)]
    pub created_at: String,
    /// When it stops working.
    #[schema(format = DateTime)]
    pub expires_at: String,
    /// When it was last used: never, yet.
    #[schema(format = DateTime)]
    pub last_used_at: Option<String>,
    /// The secret, `lp_pat_` and 43 characters: never shown again.
    pub token: String,
    /// The MCP server to register it on.
    pub mcp: McpServer,
}

impl CreatedAccessToken {
    /// The answer of `created`, for the server `mcp`.
    #[must_use]
    pub fn new(created: CreatedToken, mcp: McpServer) -> Self {
        let summary = AccessTokenSummary::from(created.record);
        Self {
            id: summary.id,
            name: summary.name,
            prefix: summary.prefix,
            scopes: summary.scopes,
            created_at: summary.created_at,
            expires_at: summary.expires_at,
            last_used_at: summary.last_used_at,
            token: created.secret.expose().to_owned(),
            mcp,
        }
    }
}

/// A token to create.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewAccessToken {
    /// Its name: 1 to 60 characters once trimmed.
    pub name: String,
    /// What it grants: at least one scope.
    pub scopes: Vec<AccessTokenScope>,
    /// Its lifetime in days: 30, 90 or 365.
    pub expires_in_days: u16,
}

impl From<NewAccessToken> for NewToken {
    fn from(request: NewAccessToken) -> Self {
        Self {
            name: request.name,
            scopes: request.scopes.into_iter().map(Into::into).collect(),
            expires_in_days: request.expires_in_days,
        }
    }
}

/// `at` in RFC 3339.
fn timestamp(at: OffsetDateTime) -> String {
    at.format(&Rfc3339).unwrap_or_default()
}
