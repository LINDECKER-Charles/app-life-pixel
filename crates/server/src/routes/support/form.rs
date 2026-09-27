//! The form of a new request: `multipart/form-data` with `category`, `message`, `context` (JSON)
//! and an optional `screenshot`, read into a submission.

use axum::extract::multipart::{Field, Multipart, MultipartError};
use axum::http::StatusCode;
use bytes::Bytes;
use life_pixel_service::support::SupportSubmission;
use serde::Serialize;
use utoipa::ToSchema;

use super::SUPPORT_BODY_LIMIT_BYTES;
use super::schema::SupportCategory;
use crate::http::problem::{Problem, codes};

/// The form a request is sent as: its description, the handler reads the parts one by one.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupportRequestForm {
    /// What it is about.
    pub category: SupportCategory,
    /// The message: 1 to 5,000 characters once trimmed.
    pub message: String,
    /// What the app attaches without asking.
    pub context: SupportRequestContext,
    /// A PNG or a JPEG of at most 4,096 pixels a side and 5,242,880 bytes, re-encoded as PNG
    /// before it is stored; an empty part counts as none.
    #[schema(value_type = Option<String>, format = Binary)]
    pub screenshot: Option<Vec<u8>>,
}

/// The context part, as JSON.
#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SupportRequestContext {
    /// The app's version.
    pub app_version: String,
    /// The app's platform: `web`, `android`…
    pub platform: String,
    /// The interface's language code.
    pub language: String,
    /// The route template of the screen the person was on, `/editor/:animationId`: never a URL
    /// with ids.
    pub screen: String,
}

/// The parts read so far; each may come once.
#[derive(Default)]
struct Parts {
    category: Option<String>,
    message: Option<String>,
    context: Option<Bytes>,
    screenshot: Option<Bytes>,
}

/// The submission of `multipart`.
///
/// # Errors
///
/// `request.too_large` beyond the route's limit; `request.malformed` for a part missing,
/// repeated, unknown or unreadable.
pub async fn read_submission(mut multipart: Multipart) -> Result<SupportSubmission, Problem> {
    let mut parts = Parts::default();
    while let Some(field) = multipart.next_field().await.map_err(unreadable)? {
        parts.accept(field).await?;
    }
    let malformed = || Problem::new(codes::REQUEST_MALFORMED);
    Ok(SupportSubmission {
        category: parts.category.ok_or_else(malformed)?,
        message: parts.message.ok_or_else(malformed)?,
        context: parts.context.ok_or_else(malformed)?.to_vec(),
        screenshot: parts.screenshot.filter(|screenshot| !screenshot.is_empty()),
    })
}

impl Parts {
    /// Reads `field` into its part.
    async fn accept(&mut self, field: Field<'_>) -> Result<(), Problem> {
        let name = field.name().unwrap_or_default().to_owned();
        match name.as_str() {
            "category" => set(&mut self.category, field.text().await),
            "message" => set(&mut self.message, field.text().await),
            "context" => set(&mut self.context, field.bytes().await),
            "screenshot" => set(&mut self.screenshot, field.bytes().await),
            _ => Err(Problem::new(codes::REQUEST_MALFORMED)),
        }
    }
}

/// Sets `part` to what was read, once.
fn set<T>(part: &mut Option<T>, read: Result<T, MultipartError>) -> Result<(), Problem> {
    let value = read.map_err(unreadable)?;
    if part.replace(value).is_some() {
        return Err(Problem::new(codes::REQUEST_MALFORMED));
    }
    Ok(())
}

/// The problem of a body that cannot be read: too large, or not a form.
fn unreadable(error: MultipartError) -> Problem {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return Problem::new(codes::REQUEST_TOO_LARGE)
            .with_param("maxBytes", SUPPORT_BODY_LIMIT_BYTES);
    }
    Problem::new(codes::REQUEST_MALFORMED)
}
