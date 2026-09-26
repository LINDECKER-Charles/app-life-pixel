//! An account's data export: a zip laid out like a local library — the desktop app opens it once
//! unzipped —, plus [`ACCOUNT_FILE`]. The library is copied into a local library in a temporary
//! folder, one document at a time, then zipped into a temporary file on a blocking thread.

mod archive;
mod staging;

use std::fs::File;

use serde::Serialize;
use time::OffsetDateTime;

use super::ports::AccountRecord;
use super::{Accounts, AccountsError};
use crate::ids::AccountId;
use crate::library::Library;
use crate::owner::Owner;

/// The file of the export holding the account itself, beside the library's `library.json`.
pub const ACCOUNT_FILE: &str = "account.json";
/// The name of an export's file, before its date.
const FILE_NAME_PREFIX: &str = "life-pixel-export-";

/// A data export, ready to be sent.
#[derive(Debug)]
pub struct DataExport {
    /// The zip, in a temporary file removed once closed, read from its start.
    pub file: File,
    /// Its size, in bytes.
    pub bytes: u64,
    /// When it was made.
    pub made_at: OffsetDateTime,
}

impl DataExport {
    /// The name the export is downloaded as: `life-pixel-export-<YYYY-MM-DD>.zip`, the day it
    /// was made, in UTC.
    #[must_use]
    pub fn file_name(&self) -> String {
        let date = self.made_at.to_offset(time::UtcOffset::UTC).date();
        let (year, month, day) = (date.year(), u8::from(date.month()), date.day());
        format!("{FILE_NAME_PREFIX}{year:04}-{month:02}-{day:02}.zip")
    }
}

/// What [`ACCOUNT_FILE`] holds.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountFile {
    email: String,
    language: String,
    plan: String,
    #[serde(with = "time::serde::rfc3339")]
    created_at: OffsetDateTime,
}

impl AccountFile {
    fn of(record: &AccountRecord) -> Self {
        Self {
            email: record.email.clone(),
            language: record.language.clone(),
            plan: record.plan.clone(),
            created_at: record.created_at,
        }
    }
}

impl Accounts {
    /// The data of the account `id`: its projects and animations from `library`, laid out like a
    /// local library, and its address, language, plan and sign-up date.
    ///
    /// # Errors
    ///
    /// `auth.unauthenticated` when the account is gone, `service.unavailable`.
    pub async fn export_data(
        &self,
        library: &Library,
        id: AccountId,
    ) -> Result<DataExport, AccountsError> {
        let record = self.ports.accounts.get(id).await?;
        let record = record.ok_or(AccountsError::Unauthenticated)?;
        let made_at = self.now();
        let staged = staging::stage(library, &Owner::Account(id)).await?;
        let account = AccountFile::of(&record);
        let written = blocking(move || archive::write(staged.path(), &account, made_at)).await?;
        let (file, bytes) = written.map_err(|error| {
            tracing::error!(%error, "a data export could not be written");
            AccountsError::Unavailable
        })?;
        Ok(DataExport {
            file,
            bytes,
            made_at,
        })
    }
}

/// The result of `work`, on tokio's blocking pool.
async fn blocking<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> Result<T, AccountsError> {
    tokio::task::spawn_blocking(work).await.map_err(|error| {
        tracing::error!(%error, "the export worker failed");
        AccountsError::Unavailable
    })
}
