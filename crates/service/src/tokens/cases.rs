//! Creating, listing, revoking and authenticating personal access tokens.

use life_pixel_core::limits::{MAX_ACTIVE_TOKENS, TOKEN_EXPIRY_DAYS, TOKEN_NAME_MAX_CHARS};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::ports::{AccessTokenRecord, FoundToken};
use super::values::{AccessTokenSecret, TokenScope};
use super::{LAST_USE_PRECISION, Tokens, TokensError};
use crate::accounts::ports::AccountStatus;
use crate::ids::AccountId;

/// A token to create, as its owner asks for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NewToken {
    /// Its name: 1 to `TOKEN_NAME_MAX_CHARS` characters, once trimmed.
    pub name: String,
    /// What it grants: at least one scope.
    pub scopes: Vec<TokenScope>,
    /// How many days it works: one of `TOKEN_EXPIRY_DAYS`.
    pub expires_in_days: u16,
}

/// A token just created: its record, and its secret, shown this once.
#[derive(Clone, Debug)]
pub struct CreatedToken {
    /// The token.
    pub record: AccessTokenRecord,
    /// Its secret, which is never shown again.
    pub secret: AccessTokenSecret,
}

/// A token that authenticated a request: whom it acts for, and what it grants.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessToken {
    /// The token's id: what its rate limit counts by.
    pub id: Uuid,
    /// The account it acts for.
    pub account: AccountId,
    /// What it grants.
    pub scopes: Vec<TokenScope>,
}

impl AccessToken {
    /// Whether the token grants `scope`.
    ///
    /// # Errors
    ///
    /// `token.scope`, with `required`, when it does not.
    pub fn require(&self, scope: TokenScope) -> Result<(), TokensError> {
        if self.scopes.contains(&scope) {
            return Ok(());
        }
        Err(TokensError::Scope { required: scope })
    }
}

impl Tokens {
    /// Creates a token for `account`: its secret is returned this once, and only its hash kept.
    ///
    /// # Errors
    ///
    /// `token.name`, `request.malformed` without a scope, `token.expiry`, `token.limit`, or
    /// `service.unavailable`.
    pub async fn create(
        &self,
        account: AccountId,
        request: NewToken,
    ) -> Result<CreatedToken, TokensError> {
        let name = checked_name(&request.name)?;
        let scopes = checked_scopes(&request.scopes)?;
        let lifetime = checked_lifetime(request.expires_in_days)?;
        let secret = AccessTokenSecret::generate();
        let now = self.ports.clock.now();
        let record = AccessTokenRecord {
            id: self.ports.ids.new_id(),
            account,
            name,
            prefix: secret.displayed_prefix(),
            scopes,
            created_at: now,
            expires_at: now + lifetime,
            last_used_at: None,
            revoked_at: None,
        };
        let limit = (&secret.hash(), MAX_ACTIVE_TOKENS);
        self.store().create(&record, limit).await?;
        Ok(CreatedToken { record, secret })
    }

    /// The active tokens of `account`, the most recently created first.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn list(&self, account: AccountId) -> Result<Vec<AccessTokenRecord>, TokensError> {
        let now = self.ports.clock.now();
        Ok(self.store().list(account, Some(now)).await?)
    }

    /// Every token of `account`, revoked and expired ones included, the most recently created
    /// first: what the internal admin API shows.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn list_all(
        &self,
        account: AccountId,
    ) -> Result<Vec<AccessTokenRecord>, TokensError> {
        Ok(self.store().list(account, None).await?)
    }

    /// Revokes the active token `id` of `account`: it stops working at once.
    ///
    /// # Errors
    ///
    /// `token.not_found`, or `service.unavailable`.
    pub async fn revoke(&self, account: AccountId, id: Uuid) -> Result<(), TokensError> {
        let now = self.ports.clock.now();
        if self.store().revoke((account, id), now).await? {
            return Ok(());
        }
        Err(TokensError::NotFound)
    }

    /// The token of the secret `text`, when it is known, active, and its account may act; its
    /// use is recorded, at most once a minute.
    ///
    /// # Errors
    ///
    /// `token.invalid`, `auth.account_suspended`, or `service.unavailable`.
    pub async fn authenticate(&self, text: &str) -> Result<AccessToken, TokensError> {
        let secret = AccessTokenSecret::parse(text).ok_or(TokensError::Invalid)?;
        let found = self.store().find(&secret.hash()).await?;
        let now = self.ports.clock.now();
        let Some(FoundToken {
            record,
            account_status,
        }) = found.filter(|found| found.record.is_active(now))
        else {
            return Err(TokensError::Invalid);
        };
        if account_status == AccountStatus::Suspended {
            return Err(TokensError::AccountSuspended);
        }
        self.record_use(&record, now).await;
        Ok(AccessToken {
            id: record.id,
            account: record.account,
            scopes: record.scopes,
        })
    }

    /// Records that `record` was used `now`, unless it was within the last minute. A failure is
    /// logged: it never refuses the call.
    async fn record_use(&self, record: &AccessTokenRecord, now: OffsetDateTime) {
        let is_recent = record
            .last_used_at
            .is_some_and(|last| now - last < LAST_USE_PRECISION);
        if is_recent {
            return;
        }
        if let Err(error) = self.store().touch(record.id, now).await {
            tracing::warn!(%error, "the token's last use was not recorded");
        }
    }
}

/// `name` trimmed, when it holds 1 to `TOKEN_NAME_MAX_CHARS` characters.
fn checked_name(name: &str) -> Result<String, TokensError> {
    let name = name.trim();
    let length = name.chars().count();
    if length == 0 || length > TOKEN_NAME_MAX_CHARS {
        return Err(TokensError::Name);
    }
    Ok(name.to_owned())
}

/// `scopes` without repetition, in the order of [`TokenScope::ALL`], when there is one.
fn checked_scopes(scopes: &[TokenScope]) -> Result<Vec<TokenScope>, TokensError> {
    let scopes: Vec<TokenScope> = TokenScope::ALL
        .into_iter()
        .filter(|scope| scopes.contains(scope))
        .collect();
    if scopes.is_empty() {
        return Err(TokensError::NoScope);
    }
    Ok(scopes)
}

/// The lifetime of `days`, when it is one of `TOKEN_EXPIRY_DAYS`.
fn checked_lifetime(days: u16) -> Result<Duration, TokensError> {
    if !TOKEN_EXPIRY_DAYS.contains(&days) {
        return Err(TokensError::Expiry);
    }
    Ok(Duration::days(i64::from(days)))
}
