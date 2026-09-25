//! Why an accounts use case failed, as a code the interface translates.

use life_pixel_core::limits::{PASSWORD_MAX_CHARS, PASSWORD_MIN_CHARS};
use serde_json::{Map, Value};
use thiserror::Error;

use super::ports::AccountStoreError;
use crate::error::{Coded, CodedError, params};

/// The error of an [`Accounts`](super::Accounts) use case.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum AccountsError {
    /// `auth.unauthenticated`: no session, or one that ended.
    #[error("not signed in")]
    Unauthenticated,
    /// `auth.invalid_credentials`: an unknown address or a wrong password, never said apart.
    #[error("invalid credentials")]
    InvalidCredentials,
    /// `auth.email_invalid`.
    #[error("invalid email address")]
    EmailInvalid,
    /// `auth.password_length`, with `min` and `max`.
    #[error("password length out of bounds")]
    PasswordLength,
    /// `auth.email_taken`: another account has the address.
    #[error("email address taken")]
    EmailTaken,
    /// `auth.token_invalid`: an emailed token unknown, used or expired.
    #[error("invalid token")]
    TokenInvalid,
    /// `auth.current_password`: the password given to confirm a change is wrong.
    #[error("wrong current password")]
    CurrentPassword,
    /// `auth.account_suspended`.
    #[error("account suspended")]
    AccountSuspended,
    /// `account.language`, with the codes `available`.
    #[error("unknown language")]
    Language {
        /// The language codes an account may choose.
        available: Vec<String>,
    },
    /// `service.unavailable`: a store or a worker failed; the detail is logged.
    #[error("service unavailable")]
    Unavailable,
}

impl From<AccountStoreError> for AccountsError {
    fn from(error: AccountStoreError) -> Self {
        match error {
            AccountStoreError::EmailTaken => Self::EmailTaken,
            AccountStoreError::Unavailable(detail) => {
                tracing::error!(%detail, "accounts storage unavailable");
                Self::Unavailable
            }
        }
    }
}

impl Coded for AccountsError {
    fn code(&self) -> &'static str {
        match self {
            Self::Unauthenticated => "auth.unauthenticated",
            Self::InvalidCredentials => "auth.invalid_credentials",
            Self::EmailInvalid => "auth.email_invalid",
            Self::PasswordLength => "auth.password_length",
            Self::EmailTaken => "auth.email_taken",
            Self::TokenInvalid => "auth.token_invalid",
            Self::CurrentPassword => "auth.current_password",
            Self::AccountSuspended => "auth.account_suspended",
            Self::Language { .. } => "account.language",
            Self::Unavailable => "service.unavailable",
        }
    }

    fn params(&self) -> Map<String, Value> {
        match self {
            Self::PasswordLength => params([
                ("min", PASSWORD_MIN_CHARS.into()),
                ("max", PASSWORD_MAX_CHARS.into()),
            ]),
            Self::Language { available } => params([("available", available.clone().into())]),
            _ => Map::new(),
        }
    }
}

impl From<AccountsError> for CodedError {
    fn from(error: AccountsError) -> Self {
        Self::of(&error)
    }
}
