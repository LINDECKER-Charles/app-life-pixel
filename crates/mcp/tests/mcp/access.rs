//! The resource template, the server's identity, and scopes refused by the policy.

use life_pixel_mcp::Scope;
use rmcp::ServiceError;
use rmcp::model::{CallToolRequestParams, ReadResourceRequestParams, ResourceContents};
use serde_json::{Value, json};
use uuid::Uuid;

use super::session::{SERVER_NAME, Session};

/// The protocol error's data of a read that must fail.
async fn read_fails(session: &Session, uri: String) -> Value {
    let read = session
        .client
        .read_resource(ReadResourceRequestParams::new(uri))
        .await;
    let Err(ServiceError::McpError(error)) = read else {
        panic!("a protocol error expected: {read:?}");
    };
    error.data.unwrap()
}

#[tokio::test]
async fn the_server_announces_its_transport_s_name_and_the_crate_s_version() {
    let session = Session::start().await;

    let info = session.client.peer_info().unwrap();

    let server = info.server_info.clone().unwrap();
    assert_eq!(server.name, SERVER_NAME);
    assert_eq!(server.version, env!("CARGO_PKG_VERSION"));
    assert!(info.capabilities.tools.is_some() && info.capabilities.resources.is_some());
}

#[tokio::test]
async fn the_resource_template_reads_the_view_without_pixels() {
    let session = Session::start().await;
    let view = session.create(2, 2).await;
    let templates = session.client.list_resource_templates(None).await.unwrap();

    let uri = format!("life-pixel://animations/{}", view["id"].as_str().unwrap());
    let read = session
        .client
        .read_resource(ReadResourceRequestParams::new(&uri))
        .await;
    let contents = read.unwrap().contents;

    let template = &templates.resource_templates[0];
    assert_eq!(template.uri_template, "life-pixel://animations/{id}");
    assert_eq!(template.mime_type.as_deref(), Some("application/json"));
    let [
        ResourceContents::TextResourceContents {
            text, mime_type, ..
        },
    ] = contents.as_slice()
    else {
        panic!("one text content expected");
    };
    assert_eq!(mime_type.as_deref(), Some("application/json"));
    assert_eq!(serde_json::from_str::<Value>(text).unwrap(), view);
}

#[tokio::test]
async fn a_resource_that_is_missing_or_malformed_fails_with_its_code() {
    let session = Session::start().await;

    let missing = read_fails(&session, format!("life-pixel://animations/{}", Uuid::nil())).await;
    let malformed = read_fails(&session, "life-pixel://animations/nope".to_owned()).await;

    assert_eq!(
        missing,
        json!({ "code": "library.animation_not_found", "params": {} })
    );
    assert_eq!(malformed["code"], "request.malformed");
}

#[tokio::test]
async fn a_tool_outside_the_caller_s_scopes_is_refused_by_the_policy() {
    let session = Session::with_scopes(&[Scope::Read]).await;
    let create = json!({ "title": "Mascot", "width": 2, "height": 2, "project_name": "Pets" });
    let export = json!({ "id": Uuid::nil(), "format": "gif", "destination": "." });

    let (write_code, write_params) = session.fails("create_animation", create).await;
    let (_, export_params) = session.fails("export", export).await;
    let listed = session.ok("list_animations", json!({})).await;

    assert_eq!(write_code, "token.scope");
    assert_eq!(write_params, json!({ "required": "write" }));
    assert_eq!(export_params, json!({ "required": "export" }));
    assert_eq!(listed["animations"], json!([]));
}

#[tokio::test]
async fn a_resource_read_needs_the_read_scope() {
    let session = Session::with_scopes(&[Scope::Write]).await;

    let data = read_fails(&session, format!("life-pixel://animations/{}", Uuid::nil())).await;

    assert_eq!(
        data,
        json!({ "code": "token.scope", "params": { "required": "read" } })
    );
}

#[tokio::test]
async fn an_unknown_tool_is_a_protocol_error() {
    let session = Session::start().await;

    let call = session
        .client
        .call_tool(CallToolRequestParams::new("paint"))
        .await;

    assert!(matches!(call, Err(ServiceError::McpError(_))), "{call:?}");
}
