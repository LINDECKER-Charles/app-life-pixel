//! The API's description, merged from the route modules: `life-pixel-server openapi` prints it
//! with sorted keys, and `crates/server/openapi.json` commits it.

use serde_json::Value;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;

use crate::http::problem::ProblemDocument;
use crate::routes::{self, health};
use crate::state::AppState;

/// Where the API lives.
pub const API_PREFIX: &str = "/api/v1";
/// The session cookie, bound to its host.
pub const SESSION_COOKIE: &str = "__Host-lp_session";
/// The header carrying a session's CSRF token.
pub const CSRF_HEADER: &str = "X-CSRF-Token";

/// The description's fixed part: its title, the problem schema and the security schemes.
#[derive(OpenApi)]
#[openapi(
    info(title = "Life Pixel API", description = "The Life Pixel server's public API."),
    components(schemas(ProblemDocument)),
    modifiers(&SecuritySchemes)
)]
struct ApiDoc;

/// The session cookie with its CSRF header, and bearer tokens.
struct SecuritySchemes;

impl Modify for SecuritySchemes {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        let session = ApiKey::Cookie(ApiKeyValue::new(SESSION_COOKIE));
        let csrf = ApiKey::Header(ApiKeyValue::new(CSRF_HEADER));
        let bearer = HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build();
        components.add_security_scheme("session", SecurityScheme::ApiKey(session));
        components.add_security_scheme("csrf", SecurityScheme::ApiKey(csrf));
        components.add_security_scheme("bearer", SecurityScheme::Http(bearer));
    }
}

/// The public description: every route group of the public router. One line per group.
#[must_use]
pub fn description() -> utoipa::openapi::OpenApi {
    let api = routes::api_rate_limited()
        .merge(routes::api_own_policies())
        .merge(routes::api_session_opening());
    OpenApiRouter::<AppState>::with_openapi(ApiDoc::openapi())
        .merge(health::router())
        .nest(API_PREFIX, api)
        .into_openapi()
}

/// The description as `openapi` prints it: JSON with sorted keys, two-space indents, and a
/// final newline, so that it compares byte for byte.
///
/// # Errors
///
/// When the description does not serialize.
pub fn to_json() -> serde_json::Result<String> {
    let value = sort_keys(serde_json::to_value(description())?);
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

/// `value` with the keys of every object sorted.
pub(crate) fn sort_keys(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries: Vec<_> = object.into_iter().collect();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, sort_keys(value)))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_keys).collect()),
        other => other,
    }
}
