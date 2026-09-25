//! `/healthz`: `200 {"status":"ok"}` when the database answers, `503` otherwise.

mod common;

use common::TestServer;
use serde_json::json;

#[tokio::test]
async fn a_server_whose_database_answers_is_healthy() {
    let answer = TestServer::new().get("/healthz").await;
    assert_eq!(answer.status, 200);
    assert_eq!(answer.json(), json!({ "status": "ok" }));
}

#[tokio::test]
async fn a_server_whose_database_does_not_answer_is_unavailable() {
    let answer = TestServer::with(|_| {}, false).get("/healthz").await;
    answer.assert_problem(503, "service.unavailable");
}
