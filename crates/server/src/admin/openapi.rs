//! The internal admin API's description: `life-pixel-server admin-openapi` prints it with sorted
//! keys, and `crates/server/admin-openapi.json` commits it for the admin server (H11).

use utoipa::openapi::security::{
    ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityRequirement, SecurityScheme,
};
use utoipa::{Modify, OpenApi};
use utoipa_axum::router::OpenApiRouter;

use super::ADMIN_API_PREFIX;
use super::auth::{ADMIN_EMAIL_HEADER, ADMIN_ID_HEADER};
use crate::http::problem::ProblemDocument;
use crate::openapi::sort_keys;
use crate::state::AppState;

/// The security schemes every route requires together.
const SCHEMES: [&str; 3] = ["adminSecret", "adminId", "adminEmail"];

/// The description's fixed part: its title, the problem schema and the security.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Life Pixel internal admin API",
        description = "The Life Pixel server's internal admin API, which only the admin server \
                       calls: every request carries `Authorization: Bearer` with \
                       `LP_ADMIN_API_SECRET`, and the acting admin's `X-Admin-Id` and \
                       `X-Admin-Email`."
    ),
    components(schemas(ProblemDocument)),
    modifiers(&AdminSecurity)
)]
struct AdminApiDoc;

/// The admin API's secret as a bearer token, and the admin's two headers, on every route.
struct AdminSecurity;

impl Modify for AdminSecurity {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        let secret = HttpBuilder::new().scheme(HttpAuthScheme::Bearer).build();
        let id = ApiKey::Header(ApiKeyValue::new(ADMIN_ID_HEADER));
        let email = ApiKey::Header(ApiKeyValue::new(ADMIN_EMAIL_HEADER));
        let [secret_name, id_name, email_name] = SCHEMES;
        components.add_security_scheme(secret_name, SecurityScheme::Http(secret));
        components.add_security_scheme(id_name, SecurityScheme::ApiKey(id));
        components.add_security_scheme(email_name, SecurityScheme::ApiKey(email));
        let requirement = SCHEMES
            .iter()
            .fold(SecurityRequirement::default(), |all, name| {
                all.add::<&str, [&str; 0], &str>(name, [])
            });
        openapi.security = Some(vec![requirement]);
    }
}

/// The description of every route of the internal admin API.
#[must_use]
pub fn description() -> utoipa::openapi::OpenApi {
    OpenApiRouter::<AppState>::with_openapi(AdminApiDoc::openapi())
        .nest(ADMIN_API_PREFIX, super::routes::routes())
        .into_openapi()
}

/// The description as `admin-openapi` prints it: JSON with sorted keys, two-space indents, and
/// a final newline, so that it compares byte for byte.
///
/// # Errors
///
/// When the description does not serialize.
pub fn to_json() -> serde_json::Result<String> {
    let value = sort_keys(serde_json::to_value(description())?);
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}
