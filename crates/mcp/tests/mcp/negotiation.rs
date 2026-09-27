//! Which MCP revision a client may settle on: only one whose results the server produces.

use rmcp::ServiceError;
use rmcp::model::{
    ClientCapabilities, ErrorCode, Implementation, ProtocolVersion, RequestMetaObject,
};

use super::session::Session;

/// The metadata of a `server/discover` probe asking for `version`.
fn probe_for(version: ProtocolVersion) -> RequestMetaObject {
    RequestMetaObject::with_client_context(
        version,
        Implementation::new("probe", "1.0.0"),
        ClientCapabilities::default(),
    )
}

/// A 2026-07-28 client must fall back to `initialize`: its list results carry cache hints the
/// server does not produce, so settling on that revision fails every `tools/list`.
#[tokio::test]
async fn a_probe_for_2026_07_28_is_refused_with_the_revisions_to_fall_back_to() {
    let session = Session::start().await;
    let probe = probe_for(ProtocolVersion::V_2026_07_28);
    let answer = session.client.discover(probe).await;
    let Err(ServiceError::McpError(refusal)) = answer else {
        panic!("2026-07-28 must be refused: {answer:?}");
    };
    assert_eq!(refusal.code, ErrorCode::UNSUPPORTED_PROTOCOL_VERSION);
    let supported = refusal.data.unwrap()["supported"].clone();
    let supported: Vec<ProtocolVersion> = serde_json::from_value(supported).unwrap();
    assert_eq!(supported.last(), Some(&ProtocolVersion::V_2025_11_25));
}
