//! The hosted caller policy: the token the layer in front of `/mcp` authenticated gives the
//! owner, and its scopes the tools allowed.

use axum::http::request::Parts;
use life_pixel_mcp::{CallerPolicy, Scope};
use life_pixel_service::tokens::{AccessToken, TokenScope, TokensError};
use life_pixel_service::{CodedError, Owner};
use rmcp::RoleServer;
use rmcp::service::RequestContext;

/// A token's account as owner, and its scopes as what it may do.
#[derive(Clone, Copy, Debug, Default)]
pub struct TokenPolicy;

impl CallerPolicy for TokenPolicy {
    fn owner(&self, context: &RequestContext<RoleServer>) -> Result<Owner, CodedError> {
        Ok(Owner::Account(caller(context)?.account))
    }

    fn allow(&self, context: &RequestContext<RoleServer>, scope: Scope) -> Result<(), CodedError> {
        Ok(caller(context)?.require(token_scope(scope))?)
    }
}

/// The token of the HTTP request behind `context`, if the layer put one there.
#[must_use]
pub fn access_token(context: &RequestContext<RoleServer>) -> Option<&AccessToken> {
    let parts = context.extensions.get::<Parts>()?;
    parts.extensions.get::<AccessToken>()
}

/// The request's token: `token.invalid` without one, which the layer prevents.
fn caller(context: &RequestContext<RoleServer>) -> Result<&AccessToken, TokensError> {
    access_token(context).ok_or(TokensError::Invalid)
}

/// The token scope granting what a tool's `scope` covers.
const fn token_scope(scope: Scope) -> TokenScope {
    match scope {
        Scope::Read => TokenScope::Read,
        Scope::Write => TokenScope::Write,
        Scope::Export => TokenScope::Export,
    }
}
