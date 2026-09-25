//! Sending the accounts' emails. The adapter renders each message from the catalogues, in the
//! account's language.

use async_trait::async_trait;
use thiserror::Error;

/// An email to one account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Email {
    /// The recipient's address.
    pub to: String,
    /// The language code to render the message in; English when the catalogues lack it.
    pub language: String,
    /// What the email says.
    pub message: Message,
}

/// What an email says. The adapter renders `email.<key>.subject` and `email.<key>.body`, with
/// `{link}` replaced by the message's link.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    /// The link that verifies the address: `<LP_PUBLIC_URL>/verify-email?token=…`.
    VerifyEmail {
        /// The link.
        link: String,
    },
    /// The link that sets a new password: `<LP_PUBLIC_URL>/reset-password/confirm?token=…`.
    ResetPassword {
        /// The link.
        link: String,
    },
    /// The password was reset or changed.
    PasswordChanged,
    /// A support request was answered (H9): the link to its thread.
    SupportReply {
        /// The link.
        link: String,
    },
}

impl Message {
    /// The message's key in the catalogues, between `email.` and `.subject` or `.body`.
    #[must_use]
    pub fn key(&self) -> &'static str {
        match self {
            Self::VerifyEmail { .. } => "verify_email",
            Self::ResetPassword { .. } => "reset_password",
            Self::PasswordChanged => "password_changed",
            Self::SupportReply { .. } => "support_reply",
        }
    }

    /// The link the message carries, if any.
    #[must_use]
    pub fn link(&self) -> Option<&str> {
        match self {
            Self::VerifyEmail { link }
            | Self::ResetPassword { link }
            | Self::SupportReply { link } => Some(link),
            Self::PasswordChanged => None,
        }
    }
}

/// Why an email could not be sent; the detail is for the logs, and never holds an address.
#[derive(Clone, Debug, PartialEq, Eq, Error)]
#[error("the email cannot be sent: {0}")]
pub struct MailError(pub String);

/// Sends emails.
#[async_trait]
pub trait Mailer: Send + Sync {
    /// Renders and sends `email`.
    async fn send(&self, email: Email) -> Result<(), MailError>;
}
