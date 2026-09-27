//! Which MCP revision a client may settle on over `/mcp`: only one whose results the server
//! produces.

use axum::http::HeaderValue;
use serde_json::json;

use crate::stack::ApiStack;
use crate::tokens::{rpc, secret};

/// JSON-RPC's code for a protocol version the server does not support.
const UNSUPPORTED_PROTOCOL_VERSION: i64 = -32022;
/// The revision the probe asks for, in its header and its `_meta` alike.
const PROBED_VERSION: &str = "2026-07-28";
/// The probe's method, repeated in the `Mcp-Method` header as 2026-07-28 requires.
const DISCOVER: &str = "server/discover";

/// A 2026-07-28 client — Claude Code among them — probes with `server/discover` and must be sent
/// back to `initialize`: settling on that revision fails every `tools/list`, whose results would
/// have to carry cache hints the server does not produce.
#[tokio::test]
async fn a_probe_for_2026_07_28_is_refused_with_the_revisions_to_fall_back_to() {
    let stack = ApiStack::new().await;
    let ada = stack.sign_up("ada@example.org").await;
    let authorization = format!("Bearer {}", secret(&stack, &ada, &["read"]).await);
    let params = json!({ "_meta": {
        "io.modelcontextprotocol/protocolVersion": PROBED_VERSION,
        "io.modelcontextprotocol/clientInfo": { "name": "probe", "version": "1.0.0" },
        "io.modelcontextprotocol/clientCapabilities": {},
    } });
    let mut probe = rpc(Some(&authorization), (DISCOVER, params));
    let headers = probe.headers_mut();
    headers.insert(
        "mcp-protocol-version",
        HeaderValue::from_static(PROBED_VERSION),
    );
    headers.insert("mcp-method", HeaderValue::from_static(DISCOVER));

    let answer = stack.send(probe).await;

    let error = answer.json()["error"].clone();
    assert_eq!(error["code"], UNSUPPORTED_PROTOCOL_VERSION, "{error}");
    let supported = error["data"]["supported"].as_array().unwrap();
    assert_eq!(supported.last().unwrap(), "2025-11-25");
}
