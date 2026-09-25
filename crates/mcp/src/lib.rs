//! The Life Pixel MCP tools, independent of their transport.
//!
//! [`LifePixelMcp`] is an `rmcp` server handler: the tools of `docs/mcp.md` and the resource
//! template `life-pixel://animations/{id}`, each mapped to a use case of `life-pixel-service`,
//! with the same validation, limits and quota as the interface. It never knows how it is served:
//! the hosted endpoint and the CLI give it the server's name, a [`CallerPolicy`] — who calls and
//! whether a tool's [`Scope`] is allowed — and an [`ExportDelivery`] — where export files go.
//!
//! Names, descriptions and schemas are in English: they address the model. Schema bounds come
//! from `core::limits`. A failure is a tool result with `isError` and one text content, the
//! JSON `{ "code": "…", "params": { … } }`; titles and names from users are returned as data.

mod caller_policy;
mod errors;
mod export_call;
mod export_delivery;
mod resources;
mod scope;
mod server;
mod tools;

pub use caller_policy::CallerPolicy;
pub use export_call::ExportCall;
pub use export_delivery::ExportDelivery;
pub use life_pixel_compiler::{ExportFile, ExportFormat};
pub use scope::Scope;
pub use server::LifePixelMcp;
