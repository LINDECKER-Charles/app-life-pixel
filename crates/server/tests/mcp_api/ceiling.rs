//! The daily ceiling: each tool call counts one for the account and the UTC day; beyond
//! `LP_PLAN_FREE_MCP_CALLS_PER_DAY`, `mcp.daily_limit` until midnight UTC.

use serde_json::json;
use time::Duration;
use time::macros::datetime;

use crate::tokens::{list_tools, result, secret, tool_call, tool_failure, tool_output};
use crate::{clock_at, stack_at};

/// The account's calls per day, oldest first.
const USAGE: &str = "select u.day::text, u.calls from mcp_usage u \
                     join accounts a on a.id = u.account_id where a.email = $1 order by u.day";

#[tokio::test]
async fn the_ceiling_refuses_calls_beyond_the_plan_until_midnight_utc() {
    let clock = clock_at(datetime!(2026-09-01 23:58 UTC));
    let two_a_day = |env: &mut std::collections::HashMap<String, String>| {
        env.insert("LP_PLAN_FREE_MCP_CALLS_PER_DAY".to_owned(), "2".to_owned());
    };
    let stack = stack_at(&clock, two_a_day).await;
    let ada = stack.sign_up("ada@example.org").await;
    let first = secret(&stack, &ada, &["read"]).await;
    let second = secret(&stack, &ada, &["read"]).await;
    let list = |secret: &str| tool_call(secret, ("list_animations", json!({})));

    tool_output(&stack.send(list(&first)).await);
    tool_output(&stack.send(list(&second)).await);
    let refused = tool_failure(&stack.send(list(&first)).await);

    let params = json!({ "limit": 2, "resetsAt": "2026-09-02T00:00:00Z" });
    assert_eq!(
        refused,
        json!({ "code": "mcp.daily_limit", "params": params })
    );
    let listing = list_tools(&first);
    result(&stack.send(listing).await);
    clock.advance(Duration::minutes(2));
    tool_output(&stack.send(list(&first)).await);
    let usage: Vec<(String, i32)> = sqlx::query_as(USAGE)
        .bind("ada@example.org")
        .fetch_all(stack.database.pool())
        .await
        .unwrap();
    assert_eq!(
        usage,
        [("2026-09-01".to_owned(), 2), ("2026-09-02".to_owned(), 1)]
    );
}
