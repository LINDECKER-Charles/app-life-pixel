//! Where the files of the `export` tool go: the transport decides.

use async_trait::async_trait;
use life_pixel_compiler::ExportFile;
use life_pixel_service::{CodedError, Owner};

use crate::export_call::ExportCall;

/// Where export files go: signed links (A3) or a directory (A4).
///
/// The `export` tool checks its own arguments, exports through the service, then hands the files
/// here; what `deliver` returns is the tool's result, as it is.
#[async_trait]
pub trait ExportDelivery: Send + Sync {
    /// The schema of the transport's own arguments of `export`: an object schema whose
    /// `properties` and `required` join those of the tool — `directory` and `overwrite` locally,
    /// nothing hosted. Those arguments reach [`deliver`](Self::deliver) in
    /// [`ExportCall::options`].
    fn export_schema(&self) -> schemars::Schema;

    /// Delivers the files of an export for `owner`, and says where they went.
    ///
    /// # Errors
    ///
    /// The transport's code: `request.malformed` for options it cannot read,
    /// `export.directory_not_allowed` locally, for example.
    #[allow(clippy::too_many_arguments)] // The signature is the contract of mcp-cli.md.
    async fn deliver(
        &self,
        owner: &Owner,
        request: ExportCall,
        files: Vec<ExportFile>,
    ) -> Result<serde_json::Value, CodedError>;
}
