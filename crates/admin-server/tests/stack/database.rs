//! The test databases: owned by `life_pixel_admin`, named `lpa_test_…`, never the real one.

use life_pixel_admin_server::testing::{TEST_DATABASE_OWNER, TEST_DATABASE_PREFIX, TestDatabase};

const OWNER_AND_ROLE: &str = "select pg_get_userbyid(datdba)::text, current_user::text \
                              from pg_database where datname = current_database()";

#[tokio::test]
async fn a_test_database_is_its_own_and_life_pixel_admin_s() {
    let database = TestDatabase::create().await.unwrap();
    assert!(database.name().starts_with(TEST_DATABASE_PREFIX));
    assert_ne!(database.name(), "life_pixel_admin");
    let (owner, role): (String, String) = sqlx::query_as(OWNER_AND_ROLE)
        .fetch_one(database.pool())
        .await
        .unwrap();
    assert_eq!(owner, TEST_DATABASE_OWNER);
    assert_eq!(role, TEST_DATABASE_OWNER);
    let tables: Vec<(String,)> = sqlx::query_as(
        "select tablename::text from pg_tables where schemaname = 'public' order by 1",
    )
    .fetch_all(database.pool())
    .await
    .unwrap();
    let names: Vec<&str> = tables.iter().map(|(name,)| name.as_str()).collect();
    assert_eq!(names, ["_sqlx_migrations", "admin_sessions", "admins"]);
    database.drop().await.unwrap();
}
