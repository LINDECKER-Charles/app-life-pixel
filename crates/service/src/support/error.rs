//! Why a support use case failed, as a code the interface translates.

use life_pixel_core::limits::{
    SCREENSHOT_MAX_BYTES, SCREENSHOT_MAX_SIDE, SUPPORT_MESSAGE_MAX_CHARS,
};
use serde_json::{Map, Value};
use thiserror::Error;

use super::ports::SupportStoreError;
use super::values::MESSAGE_MIN_CHARS;
use crate::error::{Coded, CodedError, params};

/// The error of a [`Support`](super::Support) use case.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum SupportError {
    /// `support.request_not_found`: no such request, or one of another account.
    #[error("support request not found")]
    RequestNotFound,
    /// `support.category`: a category outside the list.
    #[error("unknown support category")]
    Category,
    /// `support.message_length`, with `min` and `max`.
    #[error("support message length out of bounds")]
    MessageLength,
    /// `support.screenshot`, with `maxSide` and `maxBytes`: not a PNG or a JPEG within the
    /// limits.
    #[error("screenshot refused")]
    Screenshot,
    /// `support.request_closed`: a closed request takes no reply.
    #[error("support request closed")]
    RequestClosed,
    /// `request.malformed`: a context that does not parse, or is not what the app sends.
    #[error("malformed support request")]
    Malformed,
    /// `service.unavailable`: a store failed; the detail is logged.
    #[error("service unavailable")]
    Unavailable,
}

impl From<SupportStoreError> for SupportError {
    fn from(error: SupportStoreError) -> Self {
        tracing::error!(detail = %error.0, "support storage unavailable");
        Self::Unavailable
    }
}

impl Coded for SupportError {
    fn code(&self) -> &'static str {
        match self {
            Self::RequestNotFound => "support.request_not_found",
            Self::Category => "support.category",
            Self::MessageLength => "support.message_length",
            Self::Screenshot => "support.screenshot",
            Self::RequestClosed => "support.request_closed",
            Self::Malformed => "request.malformed",
            Self::Unavailable => "service.unavailable",
        }
    }

    fn params(&self) -> Map<String, Value> {
        match self {
            Self::MessageLength => params([
                ("min", MESSAGE_MIN_CHARS.into()),
                ("max", SUPPORT_MESSAGE_MAX_CHARS.into()),
            ]),
            Self::Screenshot => params([
                ("maxSide", SCREENSHOT_MAX_SIDE.into()),
                ("maxBytes", SCREENSHOT_MAX_BYTES.into()),
            ]),
            _ => Map::new(),
        }
    }
}

impl From<SupportError> for CodedError {
    fn from(error: SupportError) -> Self {
        Self::of(&error)
    }
}
