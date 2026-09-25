//! A client connected to the server over an in-memory duplex, and the doubles of the transport.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use life_pixel_mcp::{CallerPolicy, ExportCall, ExportDelivery, ExportFile, LifePixelMcp, Scope};
use life_pixel_service::animation::AnimationEditing;
use life_pixel_service::library::{Library, LibraryPorts};
use life_pixel_service::memory::{
    FixedClock, InMemoryLibraryStore, RecordingEvents, SequentialIds,
};
use life_pixel_service::{AccountId, CodedError, Owner, Plans};
use rmcp::model::{CallToolRequestParams, CallToolResult, ContentBlock};
use rmcp::service::{RequestContext, RunningService};
use rmcp::{RoleClient, RoleServer, ServiceExt};
use serde_json::{Map, Value, json};
use time::macros::datetime;
use uuid::Uuid;

/// The server's name in the tests.
pub const SERVER_NAME: &str = "life-pixel-test";
/// The account every call acts for.
pub const ACCOUNT: Owner = Owner::Account(AccountId::from_uuid(Uuid::from_u128(0xacc)));
/// A quota no test reaches, unless it asks for a smaller one.
const LARGE_QUOTA: u64 = 1 << 30;
/// The free plan's MCP calls a day, which the tools never count.
const MCP_CALLS_PER_DAY: u32 = 100;
/// The size of the in-memory pipe, in bytes.
const PIPE_BYTES: usize = 1 << 16;

/// Grants the scopes it holds to [`ACCOUNT`], refusing the others with `token.scope`.
pub struct ScopedPolicy {
    scopes: Vec<Scope>,
}

impl CallerPolicy for ScopedPolicy {
    fn owner(&self, _context: &RequestContext<RoleServer>) -> Result<Owner, CodedError> {
        Ok(ACCOUNT)
    }

    fn allow(&self, _context: &RequestContext<RoleServer>, scope: Scope) -> Result<(), CodedError> {
        if self.scopes.contains(&scope) {
            return Ok(());
        }
        let mut params = Map::new();
        params.insert("required".to_owned(), json!(scope.name()));
        Err(CodedError {
            code: "token.scope",
            params,
        })
    }
}

/// A file as the delivery double received it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Received {
    pub name: String,
    pub media_type: &'static str,
    pub bytes: usize,
}

/// Keeps every export it is given, and takes one argument of its own: `destination`.
#[derive(Default)]
pub struct RecordingDelivery {
    pub calls: Mutex<Vec<(ExportCall, Vec<Received>)>>,
}

#[async_trait]
impl ExportDelivery for RecordingDelivery {
    fn export_schema(&self) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": { "destination": { "type": "string" } },
            "required": ["destination"]
        })
    }

    async fn deliver(
        &self,
        _owner: &Owner,
        request: ExportCall,
        files: Vec<ExportFile>,
    ) -> Result<Value, CodedError> {
        let received: Vec<Received> = files
            .iter()
            .map(|file| Received {
                name: file.name.clone(),
                media_type: file.media_type,
                bytes: file.bytes.len(),
            })
            .collect();
        let names: Vec<&str> = received.iter().map(|file| file.name.as_str()).collect();
        let answer = json!({ "delivered": names });
        self.calls.lock().unwrap().push((request, received));
        Ok(answer)
    }
}

/// A client connected to a server acting for [`ACCOUNT`].
pub struct Session {
    pub client: RunningService<RoleClient, ()>,
    pub events: Arc<RecordingEvents>,
    pub delivery: Arc<RecordingDelivery>,
}

impl Session {
    /// A session with every scope.
    pub async fn start() -> Self {
        Self::with(&[Scope::Read, Scope::Write, Scope::Export], LARGE_QUOTA).await
    }

    /// A session granting `scopes` only.
    pub async fn with_scopes(scopes: &[Scope]) -> Self {
        Self::with(scopes, LARGE_QUOTA).await
    }

    /// A session whose account may store `quota` bytes.
    pub async fn with_quota(quota: u64) -> Self {
        Self::with(&[Scope::Read, Scope::Write, Scope::Export], quota).await
    }

    async fn with(scopes: &[Scope], quota: u64) -> Self {
        let events = Arc::new(RecordingEvents::new());
        let delivery = Arc::new(RecordingDelivery::default());
        let library = library(events.clone(), quota);
        let editing = AnimationEditing::new(library.clone(), events.clone());
        let policy = Arc::new(ScopedPolicy {
            scopes: scopes.to_vec(),
        });
        let server = LifePixelMcp::new(SERVER_NAME, (library, editing), (delivery.clone(), policy));
        let (server_io, client_io) = tokio::io::duplex(PIPE_BYTES);
        tokio::spawn(async move {
            let running = server.serve(server_io).await.unwrap();
            running.waiting().await.unwrap();
        });
        let client = ().serve(client_io).await.unwrap();
        Self {
            client,
            events,
            delivery,
        }
    }

    /// The result of the tool `name` called with `arguments`, an object.
    pub async fn call(&self, name: &'static str, arguments: Value) -> CallToolResult {
        let Value::Object(arguments) = arguments else {
            panic!("arguments must be an object");
        };
        let request = CallToolRequestParams::new(name).with_arguments(arguments);
        self.client.call_tool(request).await.unwrap()
    }

    /// The JSON result of a call that must succeed.
    pub async fn ok(&self, name: &'static str, arguments: Value) -> Value {
        let result = self.call(name, arguments).await;
        assert_eq!(result.is_error, Some(false), "{name} failed: {result:?}");
        serde_json::from_str(&only_text(&result)).unwrap()
    }

    /// The code and parameters of a call that must fail.
    pub async fn fails(&self, name: &'static str, arguments: Value) -> (String, Value) {
        let result = self.call(name, arguments).await;
        assert_eq!(result.is_error, Some(true), "{name} succeeded: {result:?}");
        let failure: Value = serde_json::from_str(&only_text(&result)).unwrap();
        let code = failure["code"].as_str().unwrap().to_owned();
        (code, failure["params"].clone())
    }

    /// The code of a call that must fail.
    pub async fn code(&self, name: &'static str, arguments: Value) -> String {
        self.fails(name, arguments).await.0
    }

    /// A blank `width × height` animation titled "Mascot" in the project "Pets": its view.
    pub async fn create(&self, width: u16, height: u16) -> Value {
        let arguments = json!({
            "title": "Mascot", "width": width, "height": height, "project_name": "Pets",
        });
        self.ok("create_animation", arguments).await
    }
}

/// A library in memory, recording its events, whose free plan stores `quota` bytes.
fn library(events: Arc<RecordingEvents>, quota: u64) -> Library {
    let ports = LibraryPorts {
        store: Arc::new(InMemoryLibraryStore::new()),
        clock: Arc::new(FixedClock::new(datetime!(2026-09-01 12:00 UTC))),
        ids: Arc::new(SequentialIds::new()),
        events,
    };
    let plans = Plans {
        free_storage_bytes: quota,
        free_mcp_calls_per_day: MCP_CALLS_PER_DAY,
    };
    Library::new(ports, plans)
}

/// The text of a result holding one text content.
pub fn only_text(result: &CallToolResult) -> String {
    let [ContentBlock::Text(text)] = result.content.as_slice() else {
        panic!("one text content expected: {result:?}");
    };
    text.text.clone()
}
