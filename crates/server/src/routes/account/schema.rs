//! The bodies of the account routes; the account itself is the one of the auth routes.

use serde::Deserialize;
use utoipa::ToSchema;

/// A change of the account: its language.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountChange {
    /// The language code of its emails and interface, one of `languages.json`.
    pub language: String,
}

/// The confirmation of a deletion.
#[derive(Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AccountDeletionRequest {
    /// The account's password, asked again.
    pub password: String,
}

/// A data export: a zip laid out like a local library, plus `account.json`.
#[derive(ToSchema)]
#[schema(value_type = String, format = Binary)]
pub struct AccountExport(pub Vec<u8>);
