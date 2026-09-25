//! The body of a problem, as the API describes and sends it.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;

/// An error, as RFC 9457 problem details with a stable code and its parameters, never a
/// sentence: the interface translates `errors.<code>`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(
    as = Problem,
    example = json!({
        "type": "urn:life-pixel:problem:quota.storage_exceeded",
        "status": 409,
        "code": "quota.storage_exceeded",
        "params": { "used": 99_000_000, "limit": 100_000_000, "requested": 2_000_000 }
    })
)]
pub struct ProblemDocument {
    /// `urn:life-pixel:problem:<code>`.
    #[serde(rename = "type")]
    pub problem_type: String,
    /// The HTTP status.
    pub status: u16,
    /// The stable code, dot-separated `snake_case` segments, domain first.
    pub code: String,
    /// The parameters of the code's message, with camelCase keys.
    pub params: Map<String, Value>,
}
