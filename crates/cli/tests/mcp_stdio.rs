//! `mcp`: serves `crates/mcp`'s tools over stdio, on a local library — `docs/v1/mcp-cli.md`'s
//! "A4 — CLI" tests.

#![allow(clippy::unwrap_used, reason = "a panic is a failed test")]

mod common;

use common::TestLibrary;
use rmcp::model::CallToolRequestParams;
use rmcp::transport::TokioChildProcess;
use rmcp::{RoleClient, ServiceExt};
use serde_json::json;
use tokio::process::Command;

async fn connect(
    library: &TestLibrary,
    allow_dir: &std::path::Path,
) -> rmcp::service::RunningService<RoleClient, ()> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_life-pixel"));
    command
        .arg("mcp")
        .arg("--library")
        .arg(library.path())
        .arg("--allow-dir")
        .arg(allow_dir);
    let transport = TokioChildProcess::new(command).unwrap();
    ().serve(transport).await.unwrap()
}

#[tokio::test]
async fn initialize_announces_the_server_as_life_pixel() {
    let library = TestLibrary::empty();
    let allow_dir = common::scratch_dir();
    let client = connect(&library, allow_dir.path()).await;

    let info = client.peer_info().unwrap();

    assert_eq!(info.server_info.clone().unwrap().name, "life-pixel");
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn tools_list_returns_every_tool() {
    let library = TestLibrary::empty();
    let allow_dir = common::scratch_dir();
    let client = connect(&library, allow_dir.path()).await;

    let tools = client.list_tools(None).await.unwrap().tools;

    let names: Vec<&str> = tools.iter().map(|tool| tool.name.as_ref()).collect();
    assert_eq!(names.len(), 11, "{names:?}");
    assert!(names.contains(&"list_animations"), "{names:?}");
    assert!(names.contains(&"export"), "{names:?}");
    client.cancel().await.unwrap();
}

#[tokio::test]
async fn a_round_trip_lists_then_exports_the_seeded_animation() {
    let library = TestLibrary::seeded("Mascot").await;
    let allow_dir = common::scratch_dir();
    let client = connect(&library, allow_dir.path()).await;

    let list_request = CallToolRequestParams::new("list_animations")
        .with_arguments(json!({}).as_object().cloned().unwrap());
    let listed = client.call_tool(list_request).await.unwrap();
    assert_eq!(listed.is_error, Some(false), "{listed:?}");

    let export_arguments = json!({
        "id": library.animation.uuid(),
        "format": "gif",
        "directory": allow_dir.path().to_str().unwrap(),
    });
    let export_request = CallToolRequestParams::new("export")
        .with_arguments(export_arguments.as_object().cloned().unwrap());
    let exported = client.call_tool(export_request).await.unwrap();

    assert_eq!(exported.is_error, Some(false), "{exported:?}");
    let entries: Vec<_> = std::fs::read_dir(allow_dir.path()).unwrap().collect();
    assert_eq!(entries.len(), 1);
    client.cancel().await.unwrap();
}
