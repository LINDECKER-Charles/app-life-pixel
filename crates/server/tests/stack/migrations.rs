//! The database on the local stack: the migrations on an empty database, the owner role, the
//! readiness of `/healthz`, and the metrics' query.

use std::str::FromStr;

use life_pixel_server::database::{DatabaseReadiness, MIGRATOR, migrate, pool_options};
use life_pixel_server::readiness::Readiness;
use life_pixel_server::storage::metrics;
use life_pixel_server::testing::{TEST_DATABASE_OWNER, TestDatabase};
use sqlx::postgres::PgConnectOptions;

/// The tables of the schema.
const TABLES: [&str; 3] = ["accounts", "animations", "projects"];
/// The indexes of the schema, beside the primary and unique keys.
const INDEXES: [&str; 4] = [
    "animations_account_updated",
    "animations_project_updated",
    "animations_title_trgm",
    "projects_account_updated",
];

const APPLIED: &str = "select count(*) from _sqlx_migrations where success";
const TABLE_OWNERS: &str = "select tablename::text, tableowner::text from pg_tables \
                            where schemaname = 'public' and tablename = any($1) order by 1";
const INDEX_NAMES: &str = "select indexname::text from pg_indexes \
                           where schemaname = 'public' and indexname = any($1) order by 1";
const EXTENSIONS: &str = "select extname::text from pg_extension \
                          where extname in ('citext', 'pg_trgm') order by 1";

#[tokio::test]
async fn the_migrations_run_on_an_empty_database_then_change_nothing() {
    let database = TestDatabase::create().await.unwrap();
    let pool = database.pool();
    let applied: i64 = sqlx::query_scalar(APPLIED).fetch_one(pool).await.unwrap();
    assert_eq!(usize::try_from(applied).unwrap(), MIGRATOR.iter().count());

    migrate(pool).await.unwrap();

    let again: i64 = sqlx::query_scalar(APPLIED).fetch_one(pool).await.unwrap();
    assert_eq!(again, applied);
    let owners: Vec<(String, String)> = sqlx::query_as(TABLE_OWNERS)
        .bind(TABLES)
        .fetch_all(pool)
        .await
        .unwrap();
    let expected: Vec<(String, String)> = TABLES
        .iter()
        .map(|table| ((*table).to_owned(), TEST_DATABASE_OWNER.to_owned()))
        .collect();
    assert_eq!(owners, expected);
    let indexes: Vec<String> = sqlx::query_scalar(INDEX_NAMES)
        .bind(INDEXES)
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(indexes, INDEXES);
    let extensions: Vec<String> = sqlx::query_scalar(EXTENSIONS)
        .fetch_all(pool)
        .await
        .unwrap();
    assert_eq!(extensions, ["citext", "pg_trgm"]);
}

#[tokio::test]
async fn usage_can_never_go_negative() {
    let database = TestDatabase::create().await.unwrap();
    let account = database.create_account().await.unwrap();

    let negative = sqlx::query("update accounts set storage_used_bytes = -1 where id = $1")
        .bind(account.uuid())
        .execute(database.pool())
        .await;

    assert!(negative.is_err(), "the check constraint holds");
}

#[tokio::test]
async fn the_pool_acts_as_the_server_role() {
    let database = TestDatabase::create().await.unwrap();

    let user: String = sqlx::query_scalar("select current_user::text")
        .fetch_one(database.pool())
        .await
        .unwrap();

    assert_eq!(user, TEST_DATABASE_OWNER);
}

#[tokio::test]
async fn the_database_is_ready_when_the_pool_answers() {
    let database = TestDatabase::create().await.unwrap();
    let closed = PgConnectOptions::from_str("postgres://nobody@127.0.0.1:1/nothing").unwrap();
    let unreachable = pool_options(1).connect_lazy_with(closed);

    assert!(
        DatabaseReadiness::new(database.pool().clone())
            .is_ready()
            .await
    );
    assert!(!DatabaseReadiness::new(unreachable).is_ready().await);
}

#[tokio::test]
async fn the_storage_metrics_read_the_usage_by_plan() {
    let database = TestDatabase::create().await.unwrap();
    database.create_account().await.unwrap();

    let recorded = metrics::record(database.pool()).await;

    assert!(recorded.is_ok(), "{recorded:?}");
}
