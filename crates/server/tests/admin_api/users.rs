//! `GET /users` and `GET /users/{id}`: the search by address, id and status, by pages, and a
//! user in detail — library counts, product events of the last 30 days, sessions, requests.

use axum::http::Method;
use life_pixel_service::AccountId;
use life_pixel_service::events::subject;
use serde_json::{Value, json};
use uuid::Uuid;

use life_pixel_server::config::{FromVariable, HmacKey};

use crate::router::SECRET;
use crate::router::auth::{with_json, without_body};
use crate::stack::{ApiStack, api};
use crate::tokens::create_token;
use crate::{UNKNOWN, admin_get, call, expect, support_request, user};

/// A product event of the subject `$1`, named `$2`, `$3` days ago.
const INSERT_EVENT: &str = "insert into product_events (name, subject, occurred_at) \
                            values ($2, $1, now() - make_interval(days => $3))";
const SUSPEND: &str = "update accounts set status = 'suspended' where id = $1";

/// The addresses of the items of `page`.
fn emails(page: &Value) -> Vec<&str> {
    let items = page["items"].as_array().unwrap();
    items
        .iter()
        .map(|item| item["email"].as_str().unwrap())
        .collect()
}

/// The page of `GET /users<query>`.
async fn users(stack: &ApiStack, query: &str) -> Value {
    expect(stack, 200, admin_get(&format!("/users{query}")))
        .await
        .json()
}

#[tokio::test]
async fn the_search_finds_users_by_address_id_and_status_newest_first() {
    let stack = ApiStack::new().await;
    let (_, ada) = user(&stack, "ada@example.org").await;
    let (_, grace) = user(&stack, "grace@example.org").await;
    user(&stack, "alan@example.net").await;
    let suspend = sqlx::query(SUSPEND).bind(Uuid::parse_str(&grace).unwrap());
    suspend.execute(stack.database.pool()).await.unwrap();

    let all = users(&stack, "").await;
    assert_eq!(
        emails(&all),
        ["alan@example.net", "grace@example.org", "ada@example.org"]
    );
    assert_eq!(
        emails(&users(&stack, "?q=EXAMPLE.ORG").await),
        ["grace@example.org", "ada@example.org"]
    );
    assert_eq!(
        emails(&users(&stack, &format!("?q={ada}")).await),
        ["ada@example.org"]
    );
    assert_eq!(emails(&users(&stack, "?q=%25").await), Vec::<&str>::new());
    assert_eq!(
        emails(&users(&stack, "?status=suspended").await),
        ["grace@example.org"]
    );
    let first = users(&stack, "?limit=2").await;
    assert_eq!(emails(&first), ["alan@example.net", "grace@example.org"]);
    let cursor = first["nextCursor"].as_str().unwrap();
    let next = users(&stack, &format!("?limit=2&cursor={cursor}")).await;
    assert_eq!(emails(&next), ["ada@example.org"]);
    assert_eq!(next["nextCursor"], Value::Null);
}

#[tokio::test]
async fn a_listed_user_has_its_profile_and_last_activity() {
    let stack = ApiStack::new().await;
    let (_, ada) = user(&stack, "ada@example.org").await;
    let listed = &users(&stack, "").await["items"][0];
    assert_eq!(listed["id"], ada);
    assert_eq!(listed["emailVerified"], false);
    assert_eq!(listed["plan"], "free");
    assert_eq!(listed["status"], "active");
    assert_eq!(listed["storageUsedBytes"], 0);
    assert!(listed["createdAt"].is_string());
    assert!(listed["lastSeenAt"].is_string());
}

#[tokio::test]
async fn the_search_refuses_what_does_not_parse() {
    let stack = ApiStack::new().await;
    for query in ["?cursor=nope", "?status=gone", "?limit=many"] {
        let answer = call(&stack, admin_get(&format!("/users{query}"))).await;
        answer.assert_problem(400, "request.malformed");
    }
}

/// Product events of the account `id`: two exports and an opening in the last 30 days, and an
/// opening before.
async fn insert_events(stack: &ApiStack, id: &str) {
    let account = AccountId::from_uuid(Uuid::parse_str(id).unwrap());
    let secret = HmacKey::from_variable(SECRET).unwrap();
    let subject = subject(secret.as_bytes(), account);
    for (name, days_ago) in [
        ("export_completed", 1),
        ("export_completed", 2),
        ("app_opened", 3),
        ("app_opened", 45),
    ] {
        let insert = sqlx::query(INSERT_EVENT)
            .bind(&subject)
            .bind(name)
            .bind(days_ago);
        insert.execute(stack.database.pool()).await.unwrap();
    }
}

#[tokio::test]
async fn a_user_in_detail_has_its_library_events_sessions_and_requests() {
    let stack = ApiStack::new().await;
    let (browser, ada) = user(&stack, "ada@example.org").await;
    let project = with_json(
        api(&browser, Method::POST, "/projects"),
        &json!({ "name": "Sprites" }),
    );
    stack.created(project).await;
    let request = support_request(&stack, &browser, ("bug", "It crashes", None)).await;
    insert_events(&stack, &ada).await;

    let detail = expect(&stack, 200, admin_get(&format!("/users/{ada}")))
        .await
        .json();

    assert_eq!(detail["id"], ada);
    assert_eq!(detail["email"], "ada@example.org");
    assert_eq!(detail["language"], "en");
    assert_eq!(detail["projectCount"], 1);
    assert_eq!(detail["animationCount"], 0);
    let events =
        json!([{ "name": "app_opened", "count": 1 }, { "name": "export_completed", "count": 2 }]);
    assert_eq!(detail["events"], events);
    assert_eq!(detail["sessions"].as_array().unwrap().len(), 1);
    assert!(detail["sessions"][0]["expiresAt"].is_string());
    assert_eq!(detail["supportRequests"][0]["id"], request);
    assert_eq!(detail["supportRequests"][0]["status"], "new");
    assert!(!detail.to_string().contains("It crashes"));
}

#[tokio::test]
async fn a_user_in_detail_has_its_tokens_revoked_ones_included_never_their_secret() {
    let stack = ApiStack::new().await;
    let (browser, ada) = user(&stack, "ada@example.org").await;
    let laptop = create_token(&stack, &browser, ("Laptop", &["read"], 30)).await;
    let ci = create_token(&stack, &browser, ("CI", &["read", "export"], 365)).await;
    let revoke = api(
        &browser,
        Method::DELETE,
        &format!("/tokens/{}", laptop["id"].as_str().unwrap()),
    );
    stack.expect(204, without_body(revoke)).await;

    let detail = expect(&stack, 200, admin_get(&format!("/users/{ada}")))
        .await
        .json();

    let tokens = detail["tokens"].as_array().unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0]["name"], "CI");
    assert_eq!(tokens[0]["prefix"], ci["prefix"]);
    assert_eq!(tokens[0]["scopes"], json!(["read", "export"]));
    assert_eq!(tokens[0]["revokedAt"], Value::Null);
    assert_eq!(tokens[1]["id"], laptop["id"]);
    assert!(tokens[1]["revokedAt"].is_string());
    for created in [&laptop, &ci] {
        let secret = created["token"].as_str().unwrap();
        assert!(!detail.to_string().contains(&secret[11..]));
    }
    assert!(tokens[0].get("tokenHash").is_none());
}

#[tokio::test]
async fn an_unknown_or_malformed_user_is_refused() {
    let stack = ApiStack::new().await;
    let unknown = call(&stack, admin_get(&format!("/users/{UNKNOWN}"))).await;
    unknown.assert_problem(404, "admin.user_not_found");
    let malformed = call(&stack, admin_get("/users/not-an-id")).await;
    malformed.assert_problem(400, "request.malformed");
}
