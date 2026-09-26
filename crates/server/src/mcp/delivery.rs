//! The hosted delivery of `export`: a signed link per file, valid 15 minutes, under
//! `GET /api/v1/exports/{link}`. Nothing is stored: the link names what to compile again.

use async_trait::async_trait;
use life_pixel_mcp::{ExportCall, ExportDelivery, ExportFile};
use life_pixel_service::mcp::{ExportLink, ExportLinks};
use life_pixel_service::{CodedError, Owner};
use serde_json::{Map, Value, json};
use time::format_description::well_known::Rfc3339;

use crate::http::problem::codes;
use crate::openapi::API_PREFIX;

/// The path of the downloads under the API, before the link.
pub const EXPORTS_PATH: &str = "/exports";

/// Signed links under the server's public URL.
#[derive(Clone)]
pub struct SignedLinkDelivery {
    links: ExportLinks,
    base: String,
}

impl SignedLinkDelivery {
    /// Links signed by `links`, under `public_url`.
    #[must_use]
    pub fn new(links: ExportLinks, public_url: &str) -> Self {
        let public_url = public_url.trim_end_matches('/');
        Self {
            links,
            base: format!("{public_url}{API_PREFIX}{EXPORTS_PATH}/"),
        }
    }

    /// The link of `file`, compiled from `request`.
    fn url(
        &self,
        request: &ExportCall,
        (file, expires_at): (&ExportFile, time::OffsetDateTime),
    ) -> String {
        let link = ExportLink {
            animation: request.id,
            version: request.version,
            format: request.format,
            tag: request.tag.clone(),
            scale: request.scale,
            file_name: file.name.clone(),
            expires_at,
        };
        format!("{}{}", self.base, self.links.sign(&link))
    }
}

#[async_trait]
impl ExportDelivery for SignedLinkDelivery {
    fn export_schema(&self) -> schemars::Schema {
        schemars::json_schema!({ "type": "object", "properties": {} })
    }

    async fn deliver(
        &self,
        _owner: &Owner,
        request: ExportCall,
        files: Vec<ExportFile>,
    ) -> Result<Value, CodedError> {
        if !request.options.is_empty() {
            return Err(CodedError {
                code: codes::REQUEST_MALFORMED,
                params: Map::new(),
            });
        }
        let expires_at = self.links.expiry();
        let files: Vec<Value> = files
            .iter()
            .map(|file| {
                json!({
                    "name": file.name,
                    "media_type": file.media_type,
                    "bytes": file.bytes.len(),
                    "url": self.url(&request, (file, expires_at)),
                })
            })
            .collect();
        let expires_at = expires_at.format(&Rfc3339).unwrap_or_default();
        Ok(json!({ "files": files, "expires_at": expires_at }))
    }
}
