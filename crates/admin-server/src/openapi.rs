//! The console's API description: the admin server's own routes, and the relayed routes of the
//! server's internal admin API — `crates/server/admin-openapi.json`, read at build time — moved
//! from `/internal/admin/v1` to `/api/admin/v1`, their bearer secret and admin headers replaced
//! by the session cookie and, for a change, the CSRF header. `life-pixel-admin-server openapi`
//! prints it with sorted keys, and `crates/admin-server/openapi.json` commits it.

use serde_json::{Map, Value, json};
use thiserror::Error;
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;

use crate::admins::routes as auth;
use crate::app::{API_PREFIX, signed_in_routes};
use crate::database;
use crate::http::problem::ProblemDocument;
use crate::http::session::cookie::SESSION_COOKIE;
use crate::http::session::csrf::CSRF_HEADER;
use crate::monitoring::logs::LogLevel;
use crate::monitoring::panels::Panel;
use crate::relay::RELAYED_PATHS;
use crate::state::AppState;

/// The internal admin API's description, as the server commits it.
const SERVER_ADMIN_API: &str = include_str!("../../server/admin-openapi.json");
/// Where the internal admin API lives on the server.
const SERVER_PREFIX: &str = "/internal/admin/v1";
/// The session cookie's security scheme.
pub const SESSION_SCHEME: &str = "adminSession";
/// The CSRF header's security scheme.
pub const CSRF_SCHEME: &str = "adminCsrf";
/// The methods that change something: they send the CSRF header.
const UNSAFE_METHODS: [&str; 4] = ["post", "put", "patch", "delete"];
const PROBLEM_REF: &str = "#/components/schemas/Problem";

/// Why the description could not be written.
#[derive(Debug, Error)]
pub enum OpenApiError {
    /// A description does not serialize, or the server's does not parse.
    #[error("the description does not serialize: {0}")]
    Json(#[from] serde_json::Error),
    /// The server's description lacks what the merge needs.
    #[error("crates/server/admin-openapi.json has no {0}")]
    Missing(&'static str),
    /// Both descriptions name a different schema alike.
    #[error("the schema {0} is both the admin server's and the server's")]
    SchemaCollision(String),
}

/// The description's fixed part: its title, the problem schema, the schemas only query
/// parameters name — utoipa does not collect them —, and the security schemes.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Life Pixel admin console API",
        description = "The admin server's API, which only the admin console calls: signing in \
                       with a password and a TOTP code, monitoring, and the server's internal \
                       admin API relayed on behalf of the signed-in admin. A session is the \
                       `__Host-lpa_session` cookie; a change also sends `X-CSRF-Token`."
    ),
    components(schemas(ProblemDocument, LogLevel, Panel)),
    modifiers(&ConsoleSecurity)
)]
struct ConsoleApiDoc;

/// The session cookie and the CSRF header.
struct ConsoleSecurity;

impl Modify for ConsoleSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        let session = ApiKey::Cookie(ApiKeyValue::new(SESSION_COOKIE));
        let csrf = ApiKey::Header(ApiKeyValue::new(CSRF_HEADER));
        components.add_security_scheme(SESSION_SCHEME, SecurityScheme::ApiKey(session));
        components.add_security_scheme(CSRF_SCHEME, SecurityScheme::ApiKey(csrf));
    }
}

/// The admin server's own description: `/healthz` and its routes under `/api/admin/v1`.
#[must_use]
pub fn own_description() -> utoipa::openapi::OpenApi {
    let api = auth::session_opening()
        .merge(auth::session_closing())
        .merge(signed_in_routes());
    OpenApiRouter::<AppState>::with_openapi(ConsoleApiDoc::openapi())
        .merge(database::health_router())
        .nest(API_PREFIX, api)
        .into_openapi()
}

/// The whole description: the admin server's own, with the relayed routes of the server's.
///
/// # Errors
///
/// When a description does not serialize or lacks its paths or schemas, or both name a
/// schema alike.
pub fn description() -> Result<Value, OpenApiError> {
    let mut own = serde_json::to_value(own_description())?;
    let server: Value = serde_json::from_str(SERVER_ADMIN_API)?;
    let relayed = relayed_paths(&server)?;
    object_at(&mut own, "paths")?.extend(relayed);
    let schemas = server
        .pointer("/components/schemas")
        .and_then(Value::as_object)
        .ok_or(OpenApiError::Missing("components.schemas"))?;
    let own_schemas = own
        .pointer_mut("/components/schemas")
        .and_then(Value::as_object_mut)
        .ok_or(OpenApiError::Missing("own components.schemas"))?;
    for (name, schema) in schemas {
        match own_schemas.get(name) {
            None => drop(own_schemas.insert(name.clone(), schema.clone())),
            Some(_) if name == "Problem" => {}
            Some(_) => return Err(OpenApiError::SchemaCollision(name.clone())),
        }
    }
    Ok(own)
}

/// The description as `openapi` prints it: JSON with sorted keys, two-space indents, and a
/// final newline, so that it compares byte for byte.
///
/// # Errors
///
/// As [`description`].
pub fn to_json() -> Result<String, OpenApiError> {
    let value = sort_keys(description()?);
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

/// The server's paths the relay serves, moved under `/api/admin/v1`, with the console's
/// security.
fn relayed_paths(server: &Value) -> Result<Map<String, Value>, OpenApiError> {
    let paths = server
        .get("paths")
        .and_then(Value::as_object)
        .ok_or(OpenApiError::Missing("paths"))?;
    let relayed = paths.iter().filter_map(|(path, item)| {
        let path = path.strip_prefix(SERVER_PREFIX)?;
        is_relayed(path).then(|| (format!("{API_PREFIX}{path}"), console_item(item)))
    });
    Ok(relayed.collect())
}

/// Whether the relay serves `path`, relative to the internal admin API.
fn is_relayed(path: &str) -> bool {
    RELAYED_PATHS.iter().any(|relayed| {
        path == *relayed
            || path
                .strip_prefix(relayed)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// A path item's operations, their security the console's: the session, and the CSRF header
/// for a change, whose refusals they may answer.
fn console_item(item: &Value) -> Value {
    let mut item = item.clone();
    if let Some(operations) = item.as_object_mut() {
        for (method, operation) in operations.iter_mut() {
            let is_unsafe = UNSAFE_METHODS.contains(&method.as_str());
            if let Some(operation) = operation.as_object_mut() {
                console_operation(operation, is_unsafe);
            }
        }
    }
    item
}

fn console_operation(operation: &mut Map<String, Value>, is_unsafe: bool) {
    let mut requirement = Map::new();
    requirement.insert(SESSION_SCHEME.to_owned(), json!([]));
    if is_unsafe {
        requirement.insert(CSRF_SCHEME.to_owned(), json!([]));
    }
    operation.insert("security".to_owned(), json!([requirement]));
    let responses = operation.entry("responses").or_insert_with(|| json!({}));
    if let Some(responses) = responses.as_object_mut() {
        responses.insert(
            "401".to_owned(),
            problem("`admin.unauthenticated`: no live session"),
        );
        if is_unsafe {
            let refused = "`admin.csrf`: another origin, or no valid `X-CSRF-Token`";
            responses.insert("403".to_owned(), problem(refused));
        }
    }
}

fn problem(description: &str) -> Value {
    json!({
        "description": description,
        "content": { "application/problem+json": { "schema": { "$ref": PROBLEM_REF } } },
    })
}

fn object_at<'a>(
    value: &'a mut Value,
    key: &'static str,
) -> Result<&'a mut Map<String, Value>, OpenApiError> {
    value
        .get_mut(key)
        .and_then(Value::as_object_mut)
        .ok_or(OpenApiError::Missing(key))
}

/// `value` with the keys of every object sorted.
fn sort_keys(value: Value) -> Value {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_relayed_paths_are_the_server_s_but_its_environment() {
        let description = description().unwrap();
        let paths = description["paths"].as_object().unwrap();
        assert!(paths.contains_key("/api/admin/v1/users/{id}/export"));
        assert!(paths.contains_key("/api/admin/v1/support-requests/{id}/messages"));
        assert!(paths.contains_key("/api/admin/v1/auth/sign-in"));
        assert!(paths.contains_key("/api/admin/v1/monitoring/overview"));
        assert!(!paths.keys().any(|path| path.contains("environment")));
        assert!(!paths.keys().any(|path| path.starts_with(SERVER_PREFIX)));
    }

    #[test]
    fn a_relayed_change_asks_for_the_session_and_the_csrf_header() {
        let description = description().unwrap();
        let suspend = &description["paths"]["/api/admin/v1/users/{id}/suspend"]["post"];
        assert_eq!(
            suspend["security"],
            json!([{ "adminSession": [], "adminCsrf": [] }])
        );
        assert!(suspend["responses"]["403"].is_object());
        let export = &description["paths"]["/api/admin/v1/users/{id}/export"]["post"];
        assert_eq!(
            export["security"],
            json!([{ "adminSession": [], "adminCsrf": [] }])
        );
        assert!(export["requestBody"].is_object());
        let users = &description["paths"]["/api/admin/v1/users"]["get"];
        assert_eq!(users["security"], json!([{ "adminSession": [] }]));
        assert!(description.get("security").is_none());
        let schemes = description["components"]["securitySchemes"]
            .as_object()
            .unwrap();
        assert_eq!(
            schemes.keys().collect::<Vec<_>>(),
            ["adminCsrf", "adminSession"]
        );
    }

    #[test]
    fn every_reference_names_a_schema_of_the_description() {
        let description = description().unwrap();
        let text = description.to_string();
        let schemas = description["components"]["schemas"].as_object().unwrap();
        let prefix = "\"$ref\":\"#/components/schemas/";
        for reference in text.split(prefix).skip(1) {
            let name = reference.split('"').next().unwrap();
            assert!(schemas.contains_key(name), "{name} is not defined");
        }
    }

    #[test]
    fn only_the_relayed_prefixes_are_relayed() {
        assert!(is_relayed("/users"));
        assert!(is_relayed("/users/{id}/export"));
        assert!(!is_relayed("/usersx"));
        assert!(!is_relayed("/environment"));
    }
}
