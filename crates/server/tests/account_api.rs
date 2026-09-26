//! The account routes over the local stack (H6): the account read and its language changed, a
//! data export opened as a local library, and a deletion leaving nothing of the account. Behind
//! the `stack-tests` feature.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

#[path = "common/mod.rs"]
mod router;
#[path = "common/stack.rs"]
mod stack;

use std::io::Cursor;
use std::path::Path;

use axum::body::Body;
use axum::http::{Method, Request};
use life_pixel_service::local::LocalLibrary;
use life_pixel_service::paging::PageRequest;
use life_pixel_service::ports::LibraryStore;
use life_pixel_service::{AnimationId, Owner};
use serde_json::{Value, json};
use time::OffsetDateTime;
use time::macros::format_description;
use uuid::Uuid;
use zip::ZipArchive;

use crate::router::auth::{Browser, CLEARED_COOKIE, PASSWORD, from_app, with_json};
use crate::router::empty;
use crate::stack::{API, ApiStack, api, get, import, sample};

/// `PATCH /api/v1/account` to `language`, for `browser`.
fn change_language(browser: &Browser, language: &str) -> Request<Body> {
    with_json(
        api(browser, Method::PATCH, "/account"),
        &json!({ "language": language }),
    )
}

/// `DELETE /api/v1/account` confirmed with `password`, for `browser`.
fn delete_account(browser: &Browser, password: &str) -> Request<Body> {
    with_json(
        api(browser, Method::DELETE, "/account"),
        &json!({ "password": password }),
    )
}

/// The account of `browser`'s session.
async fn account(stack: &ApiStack, browser: &Browser) -> Value {
    stack.expect(200, get(browser, "/account")).await.json()
}

/// A project of `browser` holding the sample animation `title`: the animation.
async fn fill(stack: &ApiStack, browser: &Browser, title: &str) -> Value {
    let project = stack.project(browser, "Sprites").await;
    stack
        .created(import(browser, &project, &sample(title)))
        .await
}

#[tokio::test]
async fn the_account_is_read_and_its_language_changed() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;

    let read = account(&stack, &ada).await;
    assert_eq!(
        (read["email"].clone(), read["language"].clone()),
        (json!("ada@example.org"), json!("en"))
    );
    assert_eq!(read["storage"]["usedBytes"], 0);
    let changed = stack.expect(200, change_language(&ada, "fr")).await.json();
    assert_eq!(
        (changed["id"].clone(), changed["language"].clone()),
        (read["id"].clone(), json!("fr"))
    );
    let unknown = stack.send(change_language(&ada, "xx")).await;
    unknown.assert_problem(422, "account.language");
    let tokenless = ada.cookie(from_app(Method::PATCH, &format!("{API}/account")));
    let tokenless = with_json(tokenless, &json!({ "language": "en" }));
    stack.send(tokenless).await.assert_problem(403, "auth.csrf");
    assert_eq!(account(&stack, &ada).await["language"], "fr");
    for path in ["/account", "/account/export"] {
        let anonymous = stack
            .send(empty(Method::GET, &format!("{API}{path}")))
            .await;
        anonymous.assert_problem(401, "auth.unauthenticated");
    }
}

/// The data export of `browser`, its headers checked: its archive.
async fn export(stack: &ApiStack, browser: &Browser) -> ZipArchive<Cursor<Vec<u8>>> {
    let export = stack.expect(200, get(browser, "/account/export")).await;
    let today = OffsetDateTime::now_utc().date();
    let today = today.format(format_description!("[year]-[month]-[day]"));
    let file_name = format!("life-pixel-export-{}.zip", today.unwrap());
    let disposition = format!("attachment; filename=\"{file_name}\"");
    assert_eq!(export.header("content-type"), "application/zip");
    assert_eq!(export.header("content-disposition"), disposition);
    let length = export.body.len().to_string();
    assert_eq!(export.header("content-length"), length);
    ZipArchive::new(Cursor::new(export.body.to_vec())).unwrap()
}

/// The `account.json` of the folder `root`.
fn account_file(root: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(root.join("account.json")).unwrap()).unwrap()
}

#[tokio::test]
async fn an_export_unzipped_is_a_local_library_with_the_account_beside_it() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let walk = fill(&stack, &ada, "Walk").await;
    let id = walk["id"].as_str().unwrap();
    let path = format!("/animations/{id}/document");
    let document = stack.expect(200, get(&ada, &path)).await.body;
    let created_at = account(&stack, &ada).await["createdAt"].clone();

    let folder = tempfile::tempdir().unwrap();
    export(&stack, &ada).await.extract(folder.path()).unwrap();

    let expected = json!({
        "email": "ada@example.org", "language": "en", "plan": "free", "createdAt": created_at
    });
    assert_eq!(account_file(folder.path()), expected);
    let local = LocalLibrary::open(folder.path()).unwrap();
    let projects = local.list_projects(&Owner::Local, PageRequest::default());
    let projects = projects.await.unwrap().items;
    let project = (projects[0].name.as_str(), projects[0].animation_count);
    assert_eq!((projects.len(), project), (1, ("Sprites", 1)));
    let animation = AnimationId::from_uuid(Uuid::parse_str(id).unwrap());
    let read = local.read_document(&Owner::Local, animation).await;
    let (record, local_document) = read.unwrap();
    assert_eq!(record.meta.title.as_str(), "Walk");
    assert_eq!(local_document, document);
}

#[tokio::test]
async fn an_empty_library_exports_its_manifest_and_the_account() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;

    let archive = export(&stack, &ada).await;

    let mut names: Vec<&str> = archive.file_names().collect();
    names.sort_unstable();
    assert_eq!(names, ["account.json", "library.json"]);
}

#[tokio::test]
async fn a_deletion_needs_the_password_and_the_token() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    fill(&stack, &ada, "Walk").await;

    let wrong = stack.send(delete_account(&ada, "not the password")).await;
    wrong.assert_problem(403, "auth.current_password");
    let tokenless = ada.cookie(from_app(Method::DELETE, &format!("{API}/account")));
    let tokenless = with_json(tokenless, &json!({ "password": PASSWORD }));
    stack.send(tokenless).await.assert_problem(403, "auth.csrf");

    let used = account(&stack, &ada).await["storage"]["usedBytes"].clone();
    assert_eq!(used, sample("Walk").len());
    assert_eq!(stack.storage.keys().await.unwrap().len(), 1);
}

/// The counts of an account's rows: one query per table.
const ACCOUNT_ROWS: [&str; 5] = [
    "select count(*) from accounts where id::text = $1",
    "select count(*) from sessions where account_id::text = $1",
    "select count(*) from email_tokens where account_id::text = $1",
    "select count(*) from projects where account_id::text = $1",
    "select count(*) from animations where account_id::text = $1",
];

/// The rows of each table that the account `id` still has.
async fn rows_of(stack: &ApiStack, id: &str) -> Vec<i64> {
    let mut rows = Vec::new();
    for query in ACCOUNT_ROWS {
        rows.push(stack.count(query, id).await);
    }
    rows
}

/// The id of the account of `browser`'s session.
async fn account_id(stack: &ApiStack, browser: &Browser) -> String {
    account(stack, browser).await["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

#[tokio::test]
async fn a_deleted_account_leaves_nothing_behind() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    fill(&stack, &ada, "Walk").await;
    let ada_id = account_id(&stack, &ada).await;
    assert_eq!(rows_of(&stack, &ada_id).await, [1; 5]);

    let deleted = stack.expect(204, delete_account(&ada, PASSWORD)).await;

    assert_eq!(deleted.header("set-cookie"), CLEARED_COOKIE);
    let after = stack.send(get(&ada, "/account")).await;
    after.assert_problem(401, "auth.unauthenticated");
    assert_eq!(rows_of(&stack, &ada_id).await, [0; 5]);
    assert_eq!(stack.storage.keys().await.unwrap(), Vec::<String>::new());
    let last = stack.events.events().pop().unwrap();
    let account = last.account.map(|id| id.to_string());
    assert_eq!((last.name, account), ("account_deleted", Some(ada_id)));
}

#[tokio::test]
async fn a_deletion_leaves_the_other_accounts_whole() {
    let stack = ApiStack::new().await;
    let (ada, bob) = (
        stack.sign_up("ada@example.org").await,
        stack.sign_up("bob@example.org").await,
    );
    fill(&stack, &ada, "Walk").await;
    let run = fill(&stack, &bob, "Run").await;
    let bob_id = account_id(&stack, &bob).await;

    stack.expect(204, delete_account(&ada, PASSWORD)).await;

    assert_eq!(rows_of(&stack, &bob_id).await, [1; 5]);
    let keys = stack.storage.keys().await.unwrap();
    assert!(keys.len() == 1 && keys[0].contains(&bob_id), "{keys:?}");
    let path = format!("/animations/{}/document", run["id"].as_str().unwrap());
    assert_eq!(
        stack.expect(200, get(&bob, &path)).await.body,
        sample("Run")
    );
}
