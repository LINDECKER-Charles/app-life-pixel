//! Who calls, locally: the local user, none to authenticate and every scope granted —
//! `docs/mcp.md`'s "Where it runs".

use life_pixel_mcp::{CallerPolicy, Scope};
use life_pixel_service::{CodedError, Owner};
use rmcp::RoleServer;
use rmcp::service::RequestContext;

/// The policy of the `mcp` command: [`Owner::Local`], every [`Scope`] allowed.
#[derive(Clone, Copy, Debug, Default)]
pub struct LocalCallerPolicy;

impl CallerPolicy for LocalCallerPolicy {
    fn owner(&self, _context: &RequestContext<RoleServer>) -> Result<Owner, CodedError> {
        Ok(Owner::Local)
    }

    fn allow(
        &self,
        _context: &RequestContext<RoleServer>,
        _scope: Scope,
    ) -> Result<(), CodedError> {
        Ok(())
    }
}
