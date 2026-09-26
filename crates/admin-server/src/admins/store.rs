//! The `admins` and `admin_sessions` tables, in the admin server's own database.

use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

use super::sessions::{SESSION_IDLE_TIMEOUT, SessionTimes, TokenHash};

const FIND_ACTIVE_ADMIN: &str = "select id, email::text, password_hash, totp_secret, \
                                 totp_last_step from admins \
                                 where email = $1::citext and disabled_at is null";
const INSERT_ADMIN: &str = "insert into admins (id, email, password_hash, totp_secret, \
                            created_at) values ($1, $2, $3, $4, $5) \
                            on conflict (email) do nothing";
const DISABLE_ADMIN: &str = "update admins set disabled_at = $2 \
                             where email = $1::citext and disabled_at is null returning id";
const DELETE_ADMIN_SESSIONS: &str = "delete from admin_sessions where admin_id = $1";
const CLAIM_TOTP_STEP: &str = "update admins set totp_last_step = $2 \
                               where id = $1 and totp_last_step < $2 and disabled_at is null";
const INSERT_SESSION: &str = "insert into admin_sessions (token_hash, admin_id, created_at, \
                              last_seen_at, expires_at) values ($1, $2, $3, $4, $5)";
const FIND_SESSION: &str = "select a.id, a.email::text, s.created_at, s.last_seen_at, \
                            s.expires_at from admin_sessions s join admins a on a.id = s.admin_id \
                            where s.token_hash = $1 and a.disabled_at is null";
const TOUCH_SESSION: &str = "update admin_sessions set last_seen_at = $2 where token_hash = $1";
const DELETE_SESSION: &str = "delete from admin_sessions where token_hash = $1";
const PURGE_SESSIONS: &str = "delete from admin_sessions where expires_at <= $1 \
                              or last_seen_at <= $2";

/// An admin who may sign in.
#[derive(Clone, Debug)]
pub struct AdminRecord {
    /// The admin's id.
    pub id: Uuid,
    /// The address, as stored.
    pub email: String,
    /// The password's PHC string.
    pub password_hash: String,
    /// The TOTP secret, sealed.
    pub totp_secret: Vec<u8>,
    /// The last TOTP step a code was accepted for.
    pub totp_last_step: i64,
}

/// An admin to create.
#[derive(Clone, Debug)]
pub struct NewAdmin {
    /// The admin's id.
    pub id: Uuid,
    /// The address.
    pub email: String,
    /// The password's PHC string.
    pub password_hash: String,
    /// The TOTP secret, sealed for `id`.
    pub totp_secret: Vec<u8>,
    /// Now.
    pub created_at: OffsetDateTime,
}

/// A session to store.
#[derive(Clone, Debug)]
pub struct NewSession {
    /// The hash of its token.
    pub hash: TokenHash,
    /// Its admin.
    pub admin_id: Uuid,
    /// When it started, was last seen, and ends at the latest.
    pub times: SessionTimes,
}

/// The admin a session acts for: what the internal admin API is told.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminIdentity {
    /// The admin's id.
    pub id: Uuid,
    /// The admin's address.
    pub email: String,
}

/// A session found by its token's hash, with its admin.
#[derive(Clone, Debug)]
pub struct SessionRecord {
    /// The admin, who is not disabled.
    pub admin: AdminIdentity,
    /// Its times.
    pub times: SessionTimes,
}

/// The admin tables of a pool.
#[derive(Clone, Debug)]
pub struct AdminStore {
    pool: PgPool,
}

type AdminRow = (Uuid, String, String, Vec<u8>, i64);
type SessionRow = (Uuid, String, OffsetDateTime, OffsetDateTime, OffsetDateTime);

impl AdminStore {
    /// The tables of the database of `pool`.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// The admin of `email`, whatever its case, unless disabled.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn find_active(&self, email: &str) -> Result<Option<AdminRecord>, sqlx::Error> {
        let row = sqlx::query_as::<_, AdminRow>(FIND_ACTIVE_ADMIN)
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(
            |(id, email, password_hash, totp_secret, totp_last_step)| AdminRecord {
                id,
                email,
                password_hash,
                totp_secret,
                totp_last_step,
            },
        ))
    }

    /// Creates `admin`: `false` when its address is taken.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn insert(&self, admin: &NewAdmin) -> Result<bool, sqlx::Error> {
        let inserted = sqlx::query(INSERT_ADMIN)
            .bind(admin.id)
            .bind(&admin.email)
            .bind(&admin.password_hash)
            .bind(&admin.totp_secret)
            .bind(admin.created_at)
            .execute(&self.pool)
            .await?;
        Ok(inserted.rows_affected() == 1)
    }

    /// Disables the admin of `email` from `now`, and ends its sessions: `false` when there is no
    /// such admin, or it is already disabled. The address is compared as `citext`: sqlx binds
    /// it as `text`, which would compare case by case.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn disable(&self, email: &str, now: OffsetDateTime) -> Result<bool, sqlx::Error> {
        let mut transaction = self.pool.begin().await?;
        let disabled = sqlx::query_scalar::<_, Uuid>(DISABLE_ADMIN)
            .bind(email)
            .bind(now)
            .fetch_optional(&mut *transaction)
            .await?;
        let Some(id) = disabled else {
            return Ok(false);
        };
        sqlx::query(DELETE_ADMIN_SESSIONS)
            .bind(id)
            .execute(&mut *transaction)
            .await?;
        transaction.commit().await?;
        Ok(true)
    }

    /// Records `step` as the admin's last TOTP step, when it comes after the one recorded:
    /// `false` when a code of that step, or a later one, was already used.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn claim_totp_step(&self, id: Uuid, step: i64) -> Result<bool, sqlx::Error> {
        let claimed = sqlx::query(CLAIM_TOTP_STEP)
            .bind(id)
            .bind(step)
            .execute(&self.pool)
            .await?;
        Ok(claimed.rows_affected() == 1)
    }

    /// Stores a new session.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn insert_session(&self, session: &NewSession) -> Result<(), sqlx::Error> {
        sqlx::query(INSERT_SESSION)
            .bind(session.hash.as_bytes().as_slice())
            .bind(session.admin_id)
            .bind(session.times.created_at)
            .bind(session.times.last_seen_at)
            .bind(session.times.expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// The session of `hash`, if it exists and its admin is not disabled; live or not.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn find_session(
        &self,
        hash: &TokenHash,
    ) -> Result<Option<SessionRecord>, sqlx::Error> {
        let row = sqlx::query_as::<_, SessionRow>(FIND_SESSION)
            .bind(hash.as_bytes().as_slice())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(
            |(id, email, created_at, last_seen_at, expires_at)| SessionRecord {
                admin: AdminIdentity { id, email },
                times: SessionTimes {
                    created_at,
                    last_seen_at,
                    expires_at,
                },
            },
        ))
    }

    /// Moves the session's `last_seen_at` to `now`.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn touch_session(
        &self,
        hash: &TokenHash,
        now: OffsetDateTime,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(TOUCH_SESSION)
            .bind(hash.as_bytes().as_slice())
            .bind(now)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Ends the session of `hash`.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn delete_session(&self, hash: &TokenHash) -> Result<(), sqlx::Error> {
        sqlx::query(DELETE_SESSION)
            .bind(hash.as_bytes().as_slice())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Deletes the sessions that ended by `now`, and says how many.
    ///
    /// # Errors
    ///
    /// When the database fails.
    pub async fn purge_sessions(&self, now: OffsetDateTime) -> Result<u64, sqlx::Error> {
        let purged = sqlx::query(PURGE_SESSIONS)
            .bind(now)
            .bind(now - SESSION_IDLE_TIMEOUT)
            .execute(&self.pool)
            .await?;
        Ok(purged.rows_affected())
    }
}
