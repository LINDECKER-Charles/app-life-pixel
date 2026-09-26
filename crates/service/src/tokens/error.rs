//! Why a token use case failed, as a code the interface translates.

use life_pixel_core::limits::{MAX_ACTIVE_TOKENS, TOKEN_EXPIRY_DAYS, TOKEN_NAME_MAX_CHARS};
use serde_json::{Map, Value, json};
use thiserror::Error;

use super::ports::TokenStoreError;
use super::values::TokenScope;
use crate::error::{Coded, CodedError, params};

/// The error of a [`Tokens`](super::Tokens) use case.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum TokensError {
    /// `token.invalid`: unknown, revoked or expired.
    #[error("invalid token")]
    Invalid,
    /// `token.scope`: the token does not grant `required`.
    #[error("the token does not grant {}", required.as_str())]
    Scope {
        /// The scope the call needs.
        required: TokenScope,
    },
    /// `token.not_found`: the account has no such active token.
    #[error("token not found")]
    NotFound,
    /// `token.name`: blank, or longer than `TOKEN_NAME_MAX_CHARS`.
    #[error("invalid token name")]
    Name,
    /// `token.expiry`: a lifetime that is not one of `TOKEN_EXPIRY_DAYS`.
    #[error("invalid token lifetime")]
    Expiry,
    /// `token.limit`: the account already has `MAX_ACTIVE_TOKENS` active tokens.
    #[error("too many active tokens")]
    Limit,
    /// `request.malformed`: a token without a scope.
    #[error("a token needs a scope")]
    NoScope,
    /// `auth.account_suspended`: the token's account is suspended.
    #[error("account suspended")]
    AccountSuspended,
    /// `service.unavailable`: the storage failed; the detail is logged.
    #[error("service unavailable")]
    Unavailable,
}

impl From<TokenStoreError> for TokensError {
    fn from(error: TokenStoreError) -> Self {
        match error {
            TokenStoreError::LimitReached => Self::Limit,
            TokenStoreError::Unavailable(detail) => {
                tracing::error!(%detail, "token storage unavailable");
                Self::Unavailable
            }
        }
    }
}

impl Coded for TokensError {
    fn code(&self) -> &'static str {
        match self {
            Self::Invalid => "token.invalid",
            Self::Scope { .. } => "token.scope",
            Self::NotFound => "token.not_found",
            Self::Name => "token.name",
            Self::Expiry => "token.expiry",
            Self::Limit => "token.limit",
            Self::NoScope => "request.malformed",
            Self::AccountSuspended => "auth.account_suspended",
            Self::Unavailable => "service.unavailable",
        }
    }

    fn params(&self) -> Map<String, Value> {
        match self {
            Self::Scope { required } => params([("required", json!(required.as_str()))]),
            Self::Name => params([("max", json!(TOKEN_NAME_MAX_CHARS))]),
            Self::Expiry => params([("allowed", json!(TOKEN_EXPIRY_DAYS))]),
            Self::Limit => params([("max", json!(MAX_ACTIVE_TOKENS))]),
            _ => Map::new(),
        }
    }
}

impl From<TokensError> for CodedError {
    fn from(error: TokensError) -> Self {
        Self::of(&error)
    }
}
