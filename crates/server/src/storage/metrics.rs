//! The storage metrics of the catalogue of docs/admin-console.md: the pool's connections, the
//! queries' durations by name — never their SQL —, the usage by plan, and the orphans swept.

use std::future::Future;
use std::time::{Duration, Instant};

use metrics::{
    Unit, counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram,
};
use sqlx::PgPool;

/// The pool's connections, by `pool` and `state`.
pub const DB_POOL_CONNECTIONS: &str = "db_pool_connections";
/// Time to run a query, by `query`: a name, never the SQL.
pub const DB_QUERY_DURATION_SECONDS: &str = "db_query_duration_seconds";
/// The bytes of the accounts' documents, by `plan`.
pub const STORAGE_USED_BYTES: &str = "storage_used_bytes";
/// The objects the sweeper deleted.
pub const STORAGE_ORPHANS_DELETED_TOTAL: &str = "storage_orphans_deleted_total";
/// The `pool` label of the server's pool: its database.
pub const POOL_NAME: &str = "life_pixel";
/// How often the pool's connections and the usage are read.
pub const STORAGE_METRICS_PERIOD: Duration = Duration::from_secs(30);
/// The buckets of the query durations, in seconds.
pub const QUERY_DURATION_BUCKETS: &[f64] = &[
    0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0,
];

/// The usage of every plan.
const USAGE_BY_PLAN: &str = "select plan, coalesce(sum(storage_used_bytes), 0)::bigint \
                             from accounts group by plan";

/// Describes the storage metrics to the recorder, once it is installed.
pub fn describe() {
    describe_gauge!(DB_POOL_CONNECTIONS, "Connections of a database pool");
    describe_histogram!(
        DB_QUERY_DURATION_SECONDS,
        Unit::Seconds,
        "Time to run a database query"
    );
    describe_gauge!(
        STORAGE_USED_BYTES,
        Unit::Bytes,
        "Bytes of the accounts' documents"
    );
    describe_counter!(
        STORAGE_ORPHANS_DELETED_TOTAL,
        "Objects no row references, deleted by the sweeper"
    );
}

/// The output of `query`, its duration recorded under the name `name`.
pub async fn timed<T>(name: &'static str, query: impl Future<Output = T>) -> T {
    let start = Instant::now();
    let output = query.await;
    histogram!(DB_QUERY_DURATION_SECONDS, "query" => name).record(start.elapsed().as_secs_f64());
    output
}

/// Counts `deleted` more orphans swept.
pub fn count_orphans_deleted(deleted: u64) {
    counter!(STORAGE_ORPHANS_DELETED_TOTAL).increment(deleted);
}

/// Records the pool's idle and busy connections, then the usage of every plan.
///
/// # Errors
///
/// When the usage cannot be read.
pub async fn record(pool: &PgPool) -> Result<(), sqlx::Error> {
    let idle = u32::try_from(pool.num_idle()).unwrap_or(u32::MAX);
    let in_use = pool.size().saturating_sub(idle);
    gauge!(DB_POOL_CONNECTIONS, "pool" => POOL_NAME, "state" => "idle").set(f64::from(idle));
    gauge!(DB_POOL_CONNECTIONS, "pool" => POOL_NAME, "state" => "in_use").set(f64::from(in_use));
    let usage = sqlx::query_as::<_, (String, i64)>(USAGE_BY_PLAN).fetch_all(pool);
    for (plan, bytes) in timed("usage_by_plan", usage).await? {
        #[allow(clippy::cast_precision_loss)] // A gauge is a float; bytes stay far below 2^53.
        gauge!(STORAGE_USED_BYTES, "plan" => plan).set(bytes as f64);
    }
    Ok(())
}
