//! `GET /metrics/product` on fixed data: H13's aggregates over the week of 2 March 2026, and the
//! support queue's measures — the open requests now, the medians over the week.

use life_pixel_service::AccountId;
use serde_json::{Value, json};

use crate::stack::ApiStack;
use crate::{admin_get, call, expect};

/// The week the metrics are read over, from a Monday to the next.
const WEEK: &str = "?from=2026-03-02T00:00:00Z&to=2026-03-09T00:00:00Z";

/// The events of three subjects `$1`, `$2` and `$3`, around the week: sign-ups, exports — the
/// third subject's first one before the week —, MCP calls.
const EVENTS: &str = "insert into product_events (name, subject, properties, occurred_at) values \
     ('signed_up', $1, '{}', '2026-03-02T10:00:00Z'), \
     ('signed_up', $2, '{}', '2026-03-02T11:00:00Z'), \
     ('signed_up', $3, '{}', '2026-03-04T09:00:00Z'), \
     ('signed_up', 'later', '{}', '2026-03-10T09:00:00Z'), \
     ('export_completed', $3, '{\"format\":\"gif\",\"source\":\"app\"}', '2026-02-20T12:00:00Z'), \
     ('export_completed', $1, '{\"format\":\"gif\",\"source\":\"app\"}', '2026-03-03T12:00:00Z'), \
     ('export_completed', $1, '{\"format\":\"gif\",\"source\":\"app\"}', '2026-03-05T12:00:00Z'), \
     ('export_completed', $2, '{\"format\":\"png_frames\",\"source\":\"mcp\"}', '2026-03-05T13:00:00Z'), \
     ('export_completed', $3, '{\"format\":\"gif\",\"source\":\"app\"}', '2026-03-06T12:00:00Z'), \
     ('mcp_tool_called', $2, '{\"tool\":\"draw\",\"outcome\":\"ok\"}', '2026-03-05T14:00:00Z'), \
     ('mcp_tool_called', $2, '{\"tool\":\"draw\",\"outcome\":\"ok\"}', '2026-03-05T15:00:00Z'), \
     ('mcp_tool_called', $2, '{\"tool\":\"draw\",\"outcome\":\"error\"}', '2026-03-05T16:00:00Z')";

/// Requests of the account `$1`: three open now, of one, three and five hours; three answered
/// in the week — after one, three and two hours —, two of them resolved in it — after three
/// hours and a day —; one answered and resolved before it.
const REQUESTS: &str = "insert into support_requests (id, account_id, category, status, context, \
     created_at, updated_at, first_response_at, resolved_at) values \
     (gen_random_uuid(), $1, 'bug', 'new', '{}', now() - interval '1 hour', now(), null, null), \
     (gen_random_uuid(), $1, 'bug', 'in_progress', '{}', now() - interval '3 hours', now(), \
      null, null), \
     (gen_random_uuid(), $1, 'bug', 'waiting_for_user', '{}', now() - interval '5 hours', \
      now(), now() - interval '4 hours', null), \
     (gen_random_uuid(), $1, 'bug', 'resolved', '{}', '2026-03-02T00:00:00Z', now(), \
      '2026-03-02T01:00:00Z', '2026-03-02T03:00:00Z'), \
     (gen_random_uuid(), $1, 'bug', 'closed', '{}', '2026-03-03T00:00:00Z', now(), \
      '2026-03-03T03:00:00Z', '2026-03-04T00:00:00Z'), \
     (gen_random_uuid(), $1, 'bug', 'resolved', '{}', '2026-03-04T00:00:00Z', now(), \
      '2026-03-04T02:00:00Z', '2026-03-10T00:00:00Z'), \
     (gen_random_uuid(), $1, 'bug', 'resolved', '{}', '2026-02-01T00:00:00Z', now(), \
      '2026-02-01T01:00:00Z', '2026-02-02T00:00:00Z')";
const LARGE: &str = "update accounts set storage_used_bytes = 2000000 where id = $1";

/// `day` at midnight UTC, with its count.
fn day(day: &str, count: i64) -> Value {
    json!({ "day": format!("{day}T00:00:00Z"), "count": count })
}

/// Two accounts, the second above a megabyte, and the fixed events and requests.
async fn fixed_data(stack: &ApiStack) {
    let pool = stack.database.pool();
    let accounts: [AccountId; 2] = [
        stack.database.create_account().await.unwrap(),
        stack.database.create_account().await.unwrap(),
    ];
    let large = sqlx::query(LARGE).bind(accounts[1].uuid());
    large.execute(pool).await.unwrap();
    let events = sqlx::query(EVENTS)
        .bind("first")
        .bind("second")
        .bind("third");
    events.execute(pool).await.unwrap();
    let requests = sqlx::query(REQUESTS).bind(accounts[0].uuid());
    requests.execute(pool).await.unwrap();
}

/// The metrics of [`WEEK`] over the fixed data.
async fn weekly_metrics() -> Value {
    let stack = ApiStack::new().await;
    fixed_data(&stack).await;
    let metrics = admin_get(&format!("/metrics/product{WEEK}"));
    expect(&stack, 200, metrics).await.json()
}

#[tokio::test]
async fn the_sign_ups_activation_and_active_accounts_are_those_of_the_period() {
    let metrics = weekly_metrics().await;

    assert_eq!(metrics["from"], "2026-03-02T00:00:00Z");
    assert_eq!(metrics["to"], "2026-03-09T00:00:00Z");
    let product = &metrics["product"];
    let signups = json!([day("2026-03-02", 2), day("2026-03-04", 1)]);
    assert_eq!(product["signupsPerDay"], signups);
    assert_eq!(product["activatedAccounts"], 2);
    let active_days = [
        ("2026-03-02", 2),
        ("2026-03-03", 1),
        ("2026-03-04", 1),
        ("2026-03-05", 2),
        ("2026-03-06", 1),
    ];
    let active_days = active_days.map(|(date, count)| day(date, count));
    assert_eq!(product["activePerDay"], Value::Array(active_days.to_vec()));
    assert_eq!(product["activePerWeek"], json!([day("2026-03-02", 3)]));
    assert_eq!(product["activePerMonth"], json!([day("2026-03-01", 3)]));
}

#[tokio::test]
async fn the_exports_mcp_calls_and_storage_are_counted_by_kind() {
    let metrics = weekly_metrics().await;

    let product = &metrics["product"];
    let exports = json!([
        { "format": "gif", "source": "app", "count": 3 },
        { "format": "png_frames", "source": "mcp", "count": 1 },
    ]);
    assert_eq!(product["exports"], exports);
    let calls = json!([
        { "tool": "draw", "outcome": "error", "count": 1 },
        { "tool": "draw", "outcome": "ok", "count": 2 },
    ]);
    assert_eq!(product["mcpCalls"], calls);
    let storage = json!([
        { "sizeClass": "lt_1k", "count": 1 },
        { "sizeClass": "ge_1m", "count": 1 },
    ]);
    assert_eq!(product["storage"], storage);
}

#[tokio::test]
async fn the_support_measures_are_the_open_requests_now_and_the_medians_of_the_period() {
    let metrics = weekly_metrics().await;

    let support = &metrics["support"];
    let open = json!([
        { "status": "new", "count": 1 },
        { "status": "in_progress", "count": 1 },
        { "status": "waiting_for_user", "count": 1 },
    ]);
    assert_eq!(support["openByStatus"], open);
    let age = support["medianAgeSeconds"].as_f64().unwrap();
    assert!((10_800.0..10_860.0).contains(&age), "{age}");
    assert_eq!(support["medianFirstResponseSeconds"], 7200.0);
    assert_eq!(support["medianResolutionSeconds"], 48_600.0);
}

#[tokio::test]
async fn without_a_period_the_metrics_cover_the_last_30_days() {
    let stack = ApiStack::new().await;
    let metrics = expect(&stack, 200, admin_get("/metrics/product"))
        .await
        .json();
    let from = time::OffsetDateTime::parse(
        metrics["from"].as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    let to = time::OffsetDateTime::parse(
        metrics["to"].as_str().unwrap(),
        &time::format_description::well_known::Rfc3339,
    )
    .unwrap();
    assert_eq!(to - from, time::Duration::days(30));
    assert_eq!(metrics["support"]["medianAgeSeconds"], Value::Null);
    assert_eq!(metrics["product"]["activatedAccounts"], 0);
}

#[tokio::test]
async fn a_period_that_does_not_parse_or_ends_before_it_starts_is_refused() {
    let stack = ApiStack::new().await;
    for query in [
        "?from=yesterday",
        "?to=2026-13-01T00:00:00Z",
        "?from=2026-03-09T00:00:00Z&to=2026-03-02T00:00:00Z",
    ] {
        let answer = call(&stack, admin_get(&format!("/metrics/product{query}"))).await;
        answer.assert_problem(400, "request.malformed");
    }
}
