//! Who calls a tool, and whether the call is allowed: the transport knows, the tools do not.

use life_pixel_service::{CodedError, Owner};
use rmcp::RoleServer;
use rmcp::service::RequestContext;

use crate::scope::Scope;

/// Who calls, and whether a tool is allowed: a token's owner and scopes (A3), or `Owner::Local`.
///
/// [`LifePixelMcp`](crate::LifePixelMcp) asks for the owner, then for the tool's scope, before
/// every tool call and every resource read; a refusal becomes the call's coded failure.
pub trait CallerPolicy: Send + Sync {
    /// The owner whose library the request reads or changes.
    ///
    /// # Errors
    ///
    /// The transport's code when the caller is not known — `token.invalid`, for example.
    fn owner(&self, context: &RequestContext<RoleServer>) -> Result<Owner, CodedError>;

    /// Whether the request may do what `scope` covers.
    ///
    /// # Errors
    ///
    /// The transport's code when it may not — `token.scope` with `required`, for example.
    fn allow(&self, context: &RequestContext<RoleServer>, scope: Scope) -> Result<(), CodedError>;
}
