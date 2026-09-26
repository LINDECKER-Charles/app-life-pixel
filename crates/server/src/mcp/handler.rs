//! The hosted MCP server: the tools of `life-pixel-mcp`, each call counted against the daily
//! ceiling first, then measured and recorded as `mcp_tool_called`.

use std::sync::Arc;
use std::time::Instant;

use life_pixel_mcp::LifePixelMcp;
use life_pixel_service::mcp::DailyCeiling;
use life_pixel_service::ports::{EventSink, ProductEvent};
use life_pixel_service::{AccountId, CodedError};
use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock,
    ListResourceTemplatesResult, ListToolsResult, PaginatedRequestParams,
    ReadResourceRequestParams, ReadResourceResponse, ServerConfig, Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde_json::json;

use super::metrics::{Outcome, record_call};
use super::policy::access_token;

/// A tool call answered; properties [`TOOL`] and [`OUTCOME`].
const MCP_TOOL_CALLED: &str = "mcp_tool_called";
/// The tool called.
const TOOL: &str = "tool";
/// `ok` or `error`.
const OUTCOME: &str = "outcome";

/// The tools of [`LifePixelMcp`], metered: every call of a known tool counts one against the
/// account's daily ceiling — refused beyond it with `mcp.daily_limit` —, and is measured and
/// recorded, whatever its outcome.
#[derive(Clone)]
pub struct HostedMcpHandler {
    tools: LifePixelMcp,
    ceiling: DailyCeiling,
    events: Arc<dyn EventSink>,
}

impl HostedMcpHandler {
    /// `tools`, metered by `ceiling`, recording to `events`.
    #[must_use]
    pub fn new(tools: LifePixelMcp, ceiling: DailyCeiling, events: Arc<dyn EventSink>) -> Self {
        Self {
            tools,
            ceiling,
            events,
        }
    }

    /// The call, once the account's ceiling allows it; a refusal is the call's failure.
    async fn metered_call(
        &self,
        account: Option<AccountId>,
        (request, context): (CallToolRequestParams, RequestContext<RoleServer>),
    ) -> Result<CallToolResponse, ErrorData> {
        if let Some(account) = account
            && let Err(refused) = self.ceiling.count_call(account).await
        {
            return Ok(failure(&refused.into()).into());
        }
        self.tools.call_tool(request, context).await
    }

    /// Records `mcp_tool_called` for `account`.
    fn record(&self, account: Option<AccountId>, (tool, outcome): (&str, Outcome)) {
        let Some(account) = account else {
            return;
        };
        self.events.record(ProductEvent {
            name: MCP_TOOL_CALLED,
            account: Some(account),
            properties: vec![
                (TOOL, tool.to_owned()),
                (OUTCOME, outcome.label().to_owned()),
            ],
        });
    }
}

impl ServerHandler for HostedMcpHandler {
    fn get_info(&self) -> ServerConfig {
        self.tools.get_info()
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        self.tools.list_tools(request, context).await
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tools.get_tool(name)
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let Some(tool) = self.tools.get_tool(&request.name) else {
            return self.tools.call_tool(request, context).await;
        };
        let start = Instant::now();
        let account = access_token(&context).map(|token| token.account);
        let response = self.metered_call(account, (request, context)).await;
        let outcome = outcome_of(&response);
        record_call(&tool.name, outcome, start.elapsed());
        self.record(account, (&tool.name, outcome));
        response
    }

    async fn list_resource_templates(
        &self,
        request: Option<PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, ErrorData> {
        self.tools.list_resource_templates(request, context).await
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        self.tools.read_resource(request, context).await
    }
}

/// `ok` for a completed call that is not an error.
fn outcome_of(response: &Result<CallToolResponse, ErrorData>) -> Outcome {
    match response {
        Ok(CallToolResponse::Complete(result)) if result.is_error != Some(true) => Outcome::Ok,
        _ => Outcome::Error,
    }
}

/// The failed tool result of `error`, as the tools answer theirs: one text content, the JSON
/// `{ "code": "…", "params": { … } }`.
fn failure(error: &CodedError) -> CallToolResult {
    let body = json!({ "code": error.code, "params": error.params });
    CallToolResult::error(vec![ContentBlock::text(body.to_string())])
}
