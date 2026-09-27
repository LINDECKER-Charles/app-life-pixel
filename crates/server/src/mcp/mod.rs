//! The hosted MCP endpoint (A3): `rmcp`'s Streamable HTTP service on `/mcp`, behind a personal
//! access token, over the tools of `life-pixel-mcp`. Each call counts against the account's
//! daily ceiling; `export` answers signed links, which `GET /api/v1/exports/{link}` serves.
//!
//! The service is stateless: no session, one JSON answer per request.

mod auth;
mod delivery;
mod handler;
pub mod metrics;
mod policy;
mod storage;

use std::sync::Arc;

use axum::Router;
use axum::middleware::from_fn_with_state;
use life_pixel_mcp::{ExportFormat, LifePixelMcp};
use life_pixel_service::animation::AnimationEditing;
use life_pixel_service::library::Library;
use life_pixel_service::mcp::{DailyCeiling, ExportDownloads, ExportLinks, McpStores};
use life_pixel_service::ports::{Clock, EventSink, ProductEvent};
use rmcp::transport::streamable_http_server::session::never::NeverSessionManager;
use rmcp::transport::{StreamableHttpServerConfig, StreamableHttpService};

use crate::config::Config;
use crate::http::problem::API_BODY_LIMIT_BYTES;
use crate::state::AppState;

pub use auth::require_token;
pub use delivery::{EXPORTS_PATH, SignedLinkDelivery};
pub use handler::HostedMcpHandler;
pub use policy::TokenPolicy;
pub use storage::{PostgresAnimationOwners, PostgresMcpUsage, hosted_stores};

/// Where the endpoint is served, on the public listener.
pub const MCP_PATH: &str = "/mcp";

/// What the endpoint uses from the rest of the state.
pub struct HostedMcpParts {
    /// The hosted library's use cases.
    pub library: Library,
    /// The daily usage and the animations' owners.
    pub stores: McpStores,
    /// The clock of the ceiling and the links.
    pub clock: Arc<dyn Clock>,
    /// Where `mcp_tool_called` and `export_completed` go.
    pub events: Arc<dyn EventSink>,
}

/// The endpoint's server and the downloads of its links: cheap to clone.
#[derive(Clone)]
pub struct HostedMcp {
    handler: HostedMcpHandler,
    links: ExportLinks,
    downloads: ExportDownloads,
}

impl HostedMcp {
    /// The endpoint named `LP_MCP_SERVER_NAME`, its links signed with `LP_EXPORT_LINK_SECRET`
    /// under `LP_PUBLIC_URL`, its ceiling the plans' calls per day.
    #[must_use]
    pub fn new(config: &Config, parts: HostedMcpParts) -> Self {
        let links = ExportLinks::new(
            config.secrets.export_link.as_bytes(),
            Arc::clone(&parts.clock),
        );
        let delivery = SignedLinkDelivery::new(links.clone(), config.public_url.as_str());
        let editing = AnimationEditing::new(parts.library.clone(), Arc::clone(&parts.events));
        let tools = LifePixelMcp::new(
            config.mcp_server_name.as_str(),
            (parts.library.clone(), editing),
            (Arc::new(delivery), Arc::new(TokenPolicy)),
        );
        let ceiling = DailyCeiling::new(parts.stores.usage, parts.clock, config.plans);
        let silent = AnimationEditing::new(parts.library.clone(), Arc::new(Silent));
        let owners = parts.stores.owners;
        let downloads = ExportDownloads::new(links.clone(), owners, (parts.library, silent));
        Self {
            handler: HostedMcpHandler::new(tools, ceiling, parts.events),
            links,
            downloads,
        }
    }

    /// The format of the valid link `text`; `None` when it is tampered with or expired.
    #[must_use]
    pub fn link_format(&self, text: &str) -> Option<ExportFormat> {
        self.links.verify(text).ok().map(|link| link.format)
    }

    /// The downloads of the signed links.
    #[must_use]
    pub fn downloads(&self) -> &ExportDownloads {
        &self.downloads
    }
}

/// `/mcp`: the Streamable HTTP service, stateless and answering JSON, behind the token layer.
/// The token replaces the `Host` check against DNS rebinding: a page cannot send it.
pub fn router(state: &AppState) -> Router<AppState> {
    let handler = state.mcp.handler.clone();
    let config = StreamableHttpServerConfig::default()
        .with_legacy_session_mode(false)
        .with_json_response(true)
        .with_sse_keep_alive(None)
        .disable_allowed_hosts()
        .with_max_request_body_bytes(API_BODY_LIMIT_BYTES);
    let sessions = Arc::new(NeverSessionManager::default());
    let service = StreamableHttpService::new(move || Ok(handler.clone()), sessions, config);
    Router::new()
        .route_service(MCP_PATH, service)
        .route_layer(from_fn_with_state(state.clone(), require_token))
}

/// The events of the downloads: none, the agent's `export` already recorded
/// `export_completed`.
struct Silent;

impl EventSink for Silent {
    fn record(&self, _event: ProductEvent) {}
}
