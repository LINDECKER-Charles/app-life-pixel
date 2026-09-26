//! Signed export links: the base64url of a JSON object — animation `a`, version `v`, format
//! `f`, tag `t`, scale `s`, file name `n`, expiry `e` —, a dot, and the base64url of its
//! HMAC-SHA256 under `LP_EXPORT_LINK_SECRET`. Nothing is stored: the link is the whole state.

use std::sync::Arc;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use life_pixel_compiler::ExportFormat;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::McpError;
use crate::ids::AnimationId;
use crate::ports::Clock;

/// How long a link works.
pub const EXPORT_LINK_LIFETIME: Duration = Duration::minutes(15);
/// What separates the payload from its signature.
const SEPARATOR: char = '.';

/// One file of an export, as a link names it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportLink {
    /// The animation.
    pub animation: AnimationId,
    /// The version of its document the file is compiled from.
    pub version: u64,
    /// The format.
    pub format: ExportFormat,
    /// The tag whose frames are exported; the whole animation when `None`.
    pub tag: Option<String>,
    /// The scale of the classic formats; the export's own when `None`.
    pub scale: Option<u8>,
    /// The file of the export the link downloads: `mascot.gif`, for example.
    pub file_name: String,
    /// When the link stops working.
    pub expires_at: OffsetDateTime,
}

/// The JSON a link carries, with its one-letter keys.
#[derive(Serialize, Deserialize)]
struct Payload {
    a: Uuid,
    v: u64,
    f: ExportFormat,
    t: Option<String>,
    s: Option<u8>,
    n: String,
    e: i64,
}

/// Signs and checks export links with one key, at the time of a clock.
#[derive(Clone)]
pub struct ExportLinks {
    key: Arc<[u8]>,
    clock: Arc<dyn Clock>,
}

impl ExportLinks {
    /// Links signed with `key`, `LP_EXPORT_LINK_SECRET`'s bytes, expiring by `clock`.
    #[must_use]
    pub fn new(key: &[u8], clock: Arc<dyn Clock>) -> Self {
        Self {
            key: Arc::from(key),
            clock,
        }
    }

    /// When a link signed now expires.
    #[must_use]
    pub fn expiry(&self) -> OffsetDateTime {
        self.clock.now() + EXPORT_LINK_LIFETIME
    }

    /// The link of `link`, signed.
    #[must_use]
    pub fn sign(&self, link: &ExportLink) -> String {
        let payload = Payload {
            a: link.animation.uuid(),
            v: link.version,
            f: link.format,
            t: link.tag.clone(),
            s: link.scale,
            n: link.file_name.clone(),
            e: link.expires_at.unix_timestamp(),
        };
        let json = serde_json::to_vec(&payload).unwrap_or_default();
        let signature = self.mac(&json).finalize().into_bytes();
        let (payload, signature) = (
            URL_SAFE_NO_PAD.encode(&json),
            URL_SAFE_NO_PAD.encode(signature),
        );
        format!("{payload}{SEPARATOR}{signature}")
    }

    /// The link `text` names, when its signature holds and it has not expired.
    ///
    /// # Errors
    ///
    /// `export.link_invalid` otherwise.
    pub fn verify(&self, text: &str) -> Result<ExportLink, McpError> {
        let (payload, signature) = text.split_once(SEPARATOR).ok_or(McpError::LinkInvalid)?;
        let json = URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| McpError::LinkInvalid)?;
        let signature = URL_SAFE_NO_PAD
            .decode(signature)
            .map_err(|_| McpError::LinkInvalid)?;
        self.mac(&json)
            .verify_slice(&signature)
            .map_err(|_| McpError::LinkInvalid)?;
        let payload: Payload = serde_json::from_slice(&json).map_err(|_| McpError::LinkInvalid)?;
        let expires_at = OffsetDateTime::from_unix_timestamp(payload.e);
        let expires_at = expires_at.map_err(|_| McpError::LinkInvalid)?;
        if expires_at <= self.clock.now() {
            return Err(McpError::LinkInvalid);
        }
        Ok(ExportLink {
            animation: AnimationId::from_uuid(payload.a),
            version: payload.v,
            format: payload.f,
            tag: payload.t,
            scale: payload.s,
            file_name: payload.n,
            expires_at,
        })
    }

    fn mac(&self, json: &[u8]) -> Hmac<Sha256> {
        let mut mac = <Hmac<Sha256> as KeyInit>::new_from_slice(&self.key)
            .unwrap_or_else(|_| unreachable!("HMAC takes a key of any length"));
        mac.update(json);
        mac
    }
}
