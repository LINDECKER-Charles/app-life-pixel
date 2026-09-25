//! `TestDatabase`: a migrated database of its own for each test, on the server of
//! `LP_TEST_DATABASE_URL`, removed when the test ends.

use std::str::FromStr;
use std::time::Duration;

use life_pixel_service::AccountId;
use sqlx::postgres::PgConnectOptions;
use sqlx::{AssertSqlSafe, Connection, PgConnection, PgPool};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use uuid::Uuid;

use super::background::run_to_end;
use super::env::{TestSetupError, variable};
use crate::database::{migrate, pool_options};

/// The prefix of every test database's name.
pub const TEST_DATABASE_PREFIX: &str = "lp_test_";
/// The role that owns the test databases, and that their pools act as: the server's.
pub const TEST_DATABASE_OWNER: &str = "life_pixel";
/// The variable naming the server, as a role that may create databases.
const SERVER_URL: &str = "LP_TEST_DATABASE_URL";
/// How long a test database lives at most: older ones are the leftovers of crashed runs.
const STALE_AFTER: Duration = Duration::from_secs(60 * 60);
/// The connections of a test's pool: tests run in parallel on one server.
const TEST_POOL_CONNECTIONS: u32 = 4;
/// The hexadecimal digits of a test database's random suffix.
const SUFFIX_DIGITS: usize = 16;

const LIST_TEST_DATABASES: &str = "select datname::text, shobj_description(oid, 'pg_database') \
                                   from pg_database where starts_with(datname, $1)";
const INSERT_ACCOUNT: &str = "insert into accounts (id, email, password_hash) values ($1, $2, $3)";
/// The password hash of the accounts `create_account` makes: no password matches it.
const UNUSABLE_PASSWORD_HASH: &str = "!";

/// A migrated database of its own, owned by [`TEST_DATABASE_OWNER`]; dropped with `drop()`, or
/// when it goes out of scope.
pub struct TestDatabase {
    name: String,
    server: PgConnectOptions,
    pool: PgPool,
    is_dropped: bool,
}

impl TestDatabase {
    /// Creates, comments and migrates `lp_test_<16 random hexadecimal digits>`, after removing
    /// the test databases more than an hour old.
    ///
    /// # Errors
    ///
    /// When `LP_TEST_DATABASE_URL` is missing, or the server refuses.
    pub async fn create() -> Result<Self, TestSetupError> {
        let server = PgConnectOptions::from_str(&variable(SERVER_URL)?)?;
        let name = format!("{TEST_DATABASE_PREFIX}{:016x}", rand::random::<u64>());
        let mut admin = PgConnection::connect_with(&server).await?;
        remove_stale(&mut admin).await?;
        create_database(&mut admin, &name).await?;
        admin.close().await?;
        let options = server
            .clone()
            .database(&name)
            .options([("role", TEST_DATABASE_OWNER)]);
        let pool = pool_options(TEST_POOL_CONNECTIONS)
            .connect_with(options)
            .await?;
        migrate(&pool).await?;
        Ok(Self {
            name,
            server,
            pool,
            is_dropped: false,
        })
    }

    /// The pool of the database, acting as [`TEST_DATABASE_OWNER`].
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// The database's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// A new account, which owns nothing yet: its address is its id at
    /// [`TEST_MAIL_DOMAIN`](super::TEST_MAIL_DOMAIN), and no password signs in to it.
    ///
    /// # Errors
    ///
    /// When the insert fails.
    pub async fn create_account(&self) -> Result<AccountId, TestSetupError> {
        let id = AccountId::from_uuid(Uuid::now_v7());
        let email = format!("lp-test-{}@{}", id.uuid().simple(), super::TEST_MAIL_DOMAIN);
        sqlx::query(INSERT_ACCOUNT)
            .bind(id.uuid())
            .bind(email)
            .bind(UNUSABLE_PASSWORD_HASH)
            .execute(&self.pool)
            .await?;
        Ok(id)
    }

    /// The URL of the database, acting as [`TEST_DATABASE_OWNER`], for a server started on it;
    /// the database is kept: `test-database drop` removes it.
    ///
    /// # Errors
    ///
    /// When `LP_TEST_DATABASE_URL` is missing.
    pub fn keep(mut self) -> Result<String, TestSetupError> {
        self.is_dropped = true;
        Ok(database_url(&variable(SERVER_URL)?, &self.name))
    }

    /// Closes the pool and drops the database.
    ///
    /// # Errors
    ///
    /// When the server refuses.
    pub async fn drop(mut self) -> Result<(), TestSetupError> {
        self.is_dropped = true;
        self.pool.close().await;
        drop_database(&self.server, &self.name).await
    }

    /// Drops the test database `url` names, as `test-database drop` does.
    ///
    /// # Errors
    ///
    /// When `url` does not name a test database, or the server refuses.
    pub async fn drop_url(url: &str) -> Result<(), TestSetupError> {
        let options = PgConnectOptions::from_str(url)?;
        let name = options.get_database().unwrap_or_default().to_owned();
        let server = PgConnectOptions::from_str(&variable(SERVER_URL)?)?;
        drop_database(&server, &name).await
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        if self.is_dropped {
            return;
        }
        let (server, name) = (self.server.clone(), self.name.clone());
        run_to_end("test database", move || async move {
            drop_database(&server, &name).await
        });
    }
}

async fn create_database(admin: &mut PgConnection, name: &str) -> Result<(), TestSetupError> {
    let created_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_default();
    let create = format!("create database {name} owner {TEST_DATABASE_OWNER}");
    sqlx::raw_sql(AssertSqlSafe(create))
        .execute(&mut *admin)
        .await?;
    let comment = format!("comment on database {name} is '{created_at}'");
    sqlx::raw_sql(AssertSqlSafe(comment)).execute(admin).await?;
    Ok(())
}

/// Drops the test databases whose comment says they are more than an hour old; never one
/// without a comment, which is being created.
async fn remove_stale(admin: &mut PgConnection) -> Result<(), TestSetupError> {
    let databases = sqlx::query_as::<_, (String, Option<String>)>(LIST_TEST_DATABASES)
        .bind(TEST_DATABASE_PREFIX)
        .fetch_all(&mut *admin)
        .await?;
    let stale_before = OffsetDateTime::now_utc() - STALE_AFTER;
    for (name, comment) in databases {
        let created_at = comment.and_then(|text| OffsetDateTime::parse(&text, &Rfc3339).ok());
        if created_at.is_some_and(|created_at| created_at < stale_before) {
            execute_drop(admin, &name).await?;
        }
    }
    Ok(())
}

async fn drop_database(server: &PgConnectOptions, name: &str) -> Result<(), TestSetupError> {
    let mut admin = PgConnection::connect_with(server).await?;
    execute_drop(&mut admin, name).await?;
    admin.close().await?;
    Ok(())
}

/// Drops the database `name`, which must be a test database's: its name is spliced into the
/// statement.
async fn execute_drop(admin: &mut PgConnection, name: &str) -> Result<(), TestSetupError> {
    if !is_test_database(name) {
        return Err(TestSetupError::NotATestDatabase(name.to_owned()));
    }
    let statement = format!("drop database if exists {name} with (force)");
    sqlx::raw_sql(AssertSqlSafe(statement))
        .execute(admin)
        .await?;
    Ok(())
}

/// Whether `name` is `lp_test_` and 16 lowercase hexadecimal digits.
fn is_test_database(name: &str) -> bool {
    name.strip_prefix(TEST_DATABASE_PREFIX)
        .is_some_and(|suffix| {
            suffix.len() == SUFFIX_DIGITS
                && suffix
                    .bytes()
                    .all(|digit| matches!(digit, b'0'..=b'9' | b'a'..=b'f'))
        })
}

/// `server_url` with its database replaced by `name`, acting as [`TEST_DATABASE_OWNER`].
fn database_url(server_url: &str, name: &str) -> String {
    let base = server_url.split(['?', '#']).next().unwrap_or_default();
    let authority_start = base.find("://").map_or(0, |scheme| scheme + "://".len());
    let path_start = base[authority_start..]
        .find('/')
        .map_or(base.len(), |slash| authority_start + slash);
    let role = format!("-c%20role%3D{TEST_DATABASE_OWNER}");
    format!("{}/{name}?options={role}", &base[..path_start])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_test_databases_are_dropped() {
        assert!(is_test_database("lp_test_0123456789abcdef"));
        for name in [
            "life_pixel",
            "lp_test_",
            "lp_test_0123456789ABCDEF",
            "lp_test_x; drop",
        ] {
            assert!(!is_test_database(name), "{name}");
        }
    }

    #[test]
    fn a_database_url_names_the_database_and_the_owner_role() {
        let server = "postgres://postgres:local@127.0.0.1:5460/postgres?sslmode=disable";
        assert_eq!(
            database_url(server, "lp_test_0123456789abcdef"),
            "postgres://postgres:local@127.0.0.1:5460/lp_test_0123456789abcdef\
             ?options=-c%20role%3Dlife_pixel"
        );
        let bare = "postgres://postgres@localhost";
        assert!(database_url(bare, "x").starts_with("postgres://postgres@localhost/x?"));
    }
}
