//! Why an admin use case failed, as a code the admin console translates.

use life_pixel_core::limits::SUPPORT_MESSAGE_MAX_CHARS;
use serde_json::{Map, Value};
use thiserror::Error;

use super::ports::AdminStoreError;
use super::values::{REASON_MAX_CHARS, REASON_MIN_CHARS};
use crate::accounts::AccountsError;
use crate::error::{Coded, CodedError, params};
use crate::support::{MESSAGE_MIN_CHARS, SupportError};

/// The error of an [`Admin`](super::Admin) use case.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum AdminError {
    /// `admin.user_not_found`: no account has the id.
    #[error("user not found")]
    UserNotFound,
    /// `admin.reason_length`, with `min` and `max`.
    #[error("reason length out of bounds")]
    ReasonLength,
    /// `support.request_not_found`: no request has the id.
    #[error("support request not found")]
    RequestNotFound,
    /// `admin.screenshot_not_found`: the request came without a screenshot.
    #[error("screenshot not found")]
    ScreenshotNotFound,
    /// `support.message_length`, with `min` and `max`.
    #[error("support message length out of bounds")]
    MessageLength,
    /// `request.malformed`: a filter, an assignee or a period that does not parse.
    #[error("malformed admin request")]
    Malformed,
    /// `service.unavailable`: a store or a worker failed; the detail is logged.
    #[error("service unavailable")]
    Unavailable,
}

impl From<AdminStoreError> for AdminError {
    fn from(error: AdminStoreError) -> Self {
        tracing::error!(detail = %error.0, "admin storage unavailable");
        Self::Unavailable
    }
}

impl From<AccountsError> for AdminError {
    fn from(error: AccountsError) -> Self {
        match error {
            AccountsError::Unauthenticated => Self::UserNotFound,
            _ => Self::Unavailable,
        }
    }
}

impl From<SupportError> for AdminError {
    fn from(error: SupportError) -> Self {
        match error {
            SupportError::MessageLength => Self::MessageLength,
            SupportError::RequestNotFound => Self::RequestNotFound,
            _ => Self::Unavailable,
        }
    }
}

impl Coded for AdminError {
    fn code(&self) -> &'static str {
        match self {
            Self::UserNotFound => "admin.user_not_found",
            Self::ReasonLength => "admin.reason_length",
            Self::RequestNotFound => "support.request_not_found",
            Self::ScreenshotNotFound => "admin.screenshot_not_found",
            Self::MessageLength => "support.message_length",
            Self::Malformed => "request.malformed",
            Self::Unavailable => "service.unavailable",
        }
    }

    fn params(&self) -> Map<String, Value> {
        match self {
            Self::ReasonLength => params([
                ("min", REASON_MIN_CHARS.into()),
                ("max", REASON_MAX_CHARS.into()),
            ]),
            Self::MessageLength => params([
                ("min", MESSAGE_MIN_CHARS.into()),
                ("max", SUPPORT_MESSAGE_MAX_CHARS.into()),
            ]),
            _ => Map::new(),
        }
    }
}

impl From<AdminError> for CodedError {
    fn from(error: AdminError) -> Self {
        Self::of(&error)
    }
}
