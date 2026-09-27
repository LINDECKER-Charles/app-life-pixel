//! `POST /api/v1/events` (H13): the app's allow-list of product events. `202` once it parses,
//! whether the event is kept or dropped for lack of room; the subject comes from the session when
//! there is one.

use std::net::IpAddr;

use axum::Json;
use axum::extract::{FromRequestParts, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
use life_pixel_service::AccountId;
use life_pixel_service::ports::ProductEvent;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use crate::accounts::SessionState;
use crate::events::{APP_VERSION_PROPERTY, LANGUAGE_PROPERTY, PLATFORM_PROPERTY};
use crate::http::client_address::{ClientAddress, peer_address};
use crate::http::problem::{Problem, ProblemDocument};
use crate::http::rate_limit::{Policy, RateKey};
use crate::state::AppState;

/// The routes with a rate limit of their own, under the CSRF check.
pub fn own_policies() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(create))
}

/// The one name the app's allow-list accepts today (`docs/v1/server.md`, H13); any other value
/// fails to deserialize, and answers `request.malformed`.
#[derive(Clone, Copy, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum EventName {
    ExportCompleted,
}

/// The format a completed export used, as `docs/v1/service.md` fixes the list.
#[derive(Clone, Copy, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
enum ExportFormat {
    Wasm,
    Gif,
    Apng,
    SpriteSheet,
    PngFrames,
}

impl ExportFormat {
    fn as_str(self) -> &'static str {
        match self {
            Self::Wasm => "wasm",
            Self::Gif => "gif",
            Self::Apng => "apng",
            Self::SpriteSheet => "sprite_sheet",
            Self::PngFrames => "png_frames",
        }
    }
}

/// `export_completed`'s properties: the format downloaded and its raw, uncompressed size.
#[derive(Clone, Debug, Deserialize, ToSchema)]
struct ExportCompletedProperties {
    format: ExportFormat,
    bytes: u64,
}

/// A product event the app reports, with the browsing context every one of them carries.
#[derive(Clone, Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
struct EventRequest {
    name: EventName,
    properties: ExportCompletedProperties,
    platform: String,
    app_version: String,
    language: String,
}

/// The client's address and the account of its session, if any: combined so that the route stays
/// within a handful of extractors.
struct EventContext {
    address: IpAddr,
    account: Option<AccountId>,
}

impl FromRequestParts<AppState> for EventContext {
    type Rejection = Problem;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Problem> {
        let ClientAddress(address) = ClientAddress::resolve(
            peer_address(&parts.extensions),
            &parts.headers,
            &state.config.trusted_proxies,
        );
        let session = SessionState::from_request_parts(parts, state).await?;
        let account = match session {
            SessionState::Live(current) => Some(current.session.account.id),
            SessionState::Absent
            | SessionState::Ended
            | SessionState::Suspended
            | SessionState::Unavailable => None,
        };
        Ok(Self { address, account })
    }
}

/// Records an allowed product event; the subject comes from the session when there is one. Once
/// the request parses, this never fails: a full channel only drops the event, counted by
/// `events_dropped_total`.
#[utoipa::path(
    post,
    path = "/events",
    tag = "events",
    operation_id = "createEvent",
    request_body = EventRequest,
    responses(
        (status = ACCEPTED, description = "Recorded, or silently dropped under load"),
        (
            status = BAD_REQUEST,
            description = "`request.malformed`: a name, property or value outside the allow-list",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
        (
            status = TOO_MANY_REQUESTS,
            description = "`rate_limit.exceeded`: 60 a minute per address",
            body = ProblemDocument,
            content_type = "application/problem+json"
        ),
    )
)]
async fn create(
    State(state): State<AppState>,
    context: EventContext,
    Json(request): Json<EventRequest>,
) -> Result<StatusCode, Problem> {
    state
        .rate_limits
        .check(Policy::Events, &RateKey::Address(context.address))?;
    let EventName::ExportCompleted = request.name;
    state.events.record(ProductEvent {
        name: "export_completed",
        account: context.account,
        properties: vec![
            ("format", request.properties.format.as_str().to_owned()),
            (
                "size",
                ProductEvent::size_class(request.properties.bytes).to_owned(),
            ),
            ("source", "app".to_owned()),
            (PLATFORM_PROPERTY, request.platform),
            (APP_VERSION_PROPERTY, request.app_version),
            (LANGUAGE_PROPERTY, request.language),
        ],
    });
    Ok(StatusCode::ACCEPTED)
}
