//! The bodies of the auth routes, and the session and account they answer with.

use life_pixel_service::accounts::Account as AccountView;
use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use utoipa::ToSchema;

/// An account as its owner sees it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// Its id.
    #[schema(format = "uuid")]
    pub id: String,
    /// Its address, as it signed up with it.
    pub email: String,
    /// Whether the address is verified.
    pub email_verified: bool,
    /// The language code of its emails and interface.
    pub language: String,
    /// Its plan: `free`.
    pub plan: String,
    /// Its documents' bytes and its quota.
    pub storage: Storage,
    /// When it signed up.
    #[schema(format = DateTime)]
    pub created_at: String,
}

/// An account's storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Storage {
    /// The bytes of its documents.
    pub used_bytes: u64,
    /// Its quota, in bytes.
    pub limit_bytes: Option<u64>,
}

/// A session: the account signed in, and the token its unsafe requests send in `X-CSRF-Token`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// The account signed in.
    pub account: Account,
    /// The CSRF token of the session.
    pub csrf_token: String,
}

impl From<&AccountView> for Account {
    fn from(account: &AccountView) -> Self {
        Self {
            id: account.id.uuid().to_string(),
            email: account.email.clone(),
            email_verified: account.is_email_verified,
            language: account.language.clone(),
            plan: account.plan.clone(),
            storage: Storage {
                used_bytes: account.storage.used_bytes,
                limit_bytes: account.storage.limit_bytes,
            },
            created_at: account.created_at.format(&Rfc3339).unwrap_or_default(),
        }
    }
}

/// A sign-up.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignUpRequest {
    /// The address; spaces around it are ignored.
    pub email: String,
    /// The password: 12 to 128 characters.
    pub password: String,
    /// The language code of the interface, one of `languages.json`.
    pub language: String,
}

/// A sign-in.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SignInRequest {
    /// The address, whatever its case.
    pub email: String,
    /// The password.
    pub password: String,
}

/// The token of an emailed verification link.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct VerifyEmailRequest {
    /// The link's `token`.
    pub token: String,
}

/// A request for a link setting a new password.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetRequest {
    /// The account's address.
    pub email: String,
}

/// A new password, with the token of the emailed link.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PasswordResetConfirmation {
    /// The link's `token`.
    pub token: String,
    /// The new password: 12 to 128 characters.
    pub password: String,
}

/// A change of password.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PasswordChangeRequest {
    /// The current password.
    pub current_password: String,
    /// The new password: 12 to 128 characters.
    pub new_password: String,
}
