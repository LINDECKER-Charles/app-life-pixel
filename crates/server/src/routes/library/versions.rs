//! Versions as entity tags: an animation's `ETag` is its version, `"4"`, and a write names the
//! version it replaces in `If-Match`.

use axum::http::header::{ETAG, IF_MATCH};
use axum::http::{HeaderMap, HeaderName, HeaderValue};

use crate::http::problem::{Problem, codes};

/// The `ETag` of the document at `version`.
#[must_use]
pub fn etag(version: u64) -> [(HeaderName, HeaderValue); 1] {
    let tag = HeaderValue::try_from(format!("\"{version}\""));
    [(ETAG, tag.unwrap_or(HeaderValue::from_static("\"\"")))]
}

/// The version the request's `If-Match` names.
///
/// # Errors
///
/// `document.version_required` without `If-Match`, `request.malformed` for anything else than a
/// strong tag of a version.
pub fn required(headers: &HeaderMap) -> Result<u64, Problem> {
    let Some(value) = headers.get(IF_MATCH) else {
        return Err(Problem::new(codes::DOCUMENT_VERSION_REQUIRED));
    };
    let version = value.to_str().ok().and_then(version_of);
    version.ok_or_else(|| Problem::new(codes::REQUEST_MALFORMED))
}

/// The version of the strong tag `tag`, `"4"`.
fn version_of(tag: &str) -> Option<u64> {
    let quoted = tag.trim().strip_prefix('"')?.strip_suffix('"')?;
    quoted.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_if_match(value: &'static str) -> HeaderMap {
        HeaderMap::from_iter([(IF_MATCH, HeaderValue::from_static(value))])
    }

    #[test]
    fn a_version_travels_as_a_strong_tag() {
        assert_eq!(etag(4)[0].1, "\"4\"");
        assert_eq!(required(&with_if_match("\"4\"")), Ok(4));
        assert_eq!(required(&with_if_match(" \"12\" ")), Ok(12));
    }

    #[test]
    fn a_missing_or_unreadable_tag_is_refused() {
        let missing = required(&HeaderMap::new()).unwrap_err();
        assert_eq!(missing.code, codes::DOCUMENT_VERSION_REQUIRED);
        for value in ["4", "W/\"4\"", "*", "\"four\"", "\"4\", \"5\""] {
            let refused = required(&with_if_match(value)).unwrap_err();
            assert_eq!(refused.code, codes::REQUEST_MALFORMED, "{value}");
        }
    }
}
