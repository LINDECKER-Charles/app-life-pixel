//! Lists by cursor: they run from the most recently updated item, ties broken by id, and a page
//! resumes after the last item of the one before.

use std::fmt;
use std::str::FromStr;

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use life_pixel_core::limits::{PAGE_SIZE_DEFAULT, PAGE_SIZE_MAX};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::Coded;

/// The smallest page.
const PAGE_SIZE_MIN: u16 = 1;

/// Which page of a list to read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageRequest {
    /// The last item of the previous page; `None` for the first page.
    pub cursor: Option<Cursor>,
    /// The most items of the page.
    pub limit: u16,
}

impl PageRequest {
    /// The page after `cursor`, of `limit` items bounded to `PAGE_SIZE_MAX`, or
    /// `PAGE_SIZE_DEFAULT` when `None`.
    #[must_use]
    pub fn new(cursor: Option<Cursor>, limit: Option<u16>) -> Self {
        let max = u16::try_from(PAGE_SIZE_MAX).unwrap_or(u16::MAX);
        let default = u16::try_from(PAGE_SIZE_DEFAULT).unwrap_or(max);
        let limit = limit.unwrap_or(default).clamp(PAGE_SIZE_MIN, max);
        Self { cursor, limit }
    }
}

impl Default for PageRequest {
    fn default() -> Self {
        Self::new(None, None)
    }
}

/// A page of a list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page<T> {
    /// The items, from the most recently updated.
    pub items: Vec<T>,
    /// Where the next page starts; `None` on the last page.
    pub next_cursor: Option<Cursor>,
}

/// Lists run from the most recently updated; the cursor is the last item seen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cursor {
    /// When the item was last updated.
    pub updated_at: OffsetDateTime,
    /// The item's id.
    pub id: Uuid,
}

/// A cursor that does not decode: `request.malformed`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Error)]
#[error("malformed cursor")]
pub struct MalformedCursor;

impl Coded for MalformedCursor {
    fn code(&self) -> &'static str {
        "request.malformed"
    }

    fn params(&self) -> Map<String, Value> {
        Map::new()
    }
}

/// How a cursor travels: `{"u": <RFC 3339>, "i": <uuid>}`, then base64url.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CursorWire {
    #[serde(with = "time::serde::rfc3339")]
    u: OffsetDateTime,
    i: Uuid,
}

impl fmt::Display for Cursor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let wire = CursorWire {
            u: self.updated_at,
            i: self.id,
        };
        let json = serde_json::to_vec(&wire).map_err(|_| fmt::Error)?;
        formatter.write_str(&URL_SAFE_NO_PAD.encode(json))
    }
}

impl FromStr for Cursor {
    type Err = MalformedCursor;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let json = URL_SAFE_NO_PAD.decode(text).map_err(|_| MalformedCursor)?;
        let wire: CursorWire = serde_json::from_slice(&json).map_err(|_| MalformedCursor)?;
        Ok(Self {
            updated_at: wire.u,
            id: wire.i,
        })
    }
}

/// The page `request` asks for among `items`, in list order — from the most recently updated,
/// ties broken by the larger id —, each item placed by `cursor_of`. For adapters that list in
/// memory.
pub fn paginate<T>(
    mut items: Vec<T>,
    request: &PageRequest,
    cursor_of: impl Fn(&T) -> Cursor,
) -> Page<T> {
    items.sort_by_key(|item| std::cmp::Reverse(cursor_of(item)));
    let after = request.cursor;
    let mut items: Vec<T> = items
        .into_iter()
        .filter(|item| after.is_none_or(|after| cursor_of(item) < after))
        .collect();
    let limit = usize::from(request.limit.max(PAGE_SIZE_MIN));
    let has_more = items.len() > limit;
    items.truncate(limit);
    let next_cursor = items.last().filter(|_| has_more).map(&cursor_of);
    Page { items, next_cursor }
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::*;

    fn cursor(seconds: i64, id: u128) -> Cursor {
        Cursor {
            updated_at: datetime!(2026-01-01 0:00 UTC) + time::Duration::seconds(seconds),
            id: Uuid::from_u128(id),
        }
    }

    #[test]
    fn a_cursor_travels_as_base64url_json() {
        let cursor = Cursor {
            updated_at: datetime!(2026-03-04 05:06:07.123456 UTC),
            id: Uuid::from_u128(7),
        };
        let text = cursor.to_string();
        let json = URL_SAFE_NO_PAD.decode(&text).unwrap();
        let value: Value = serde_json::from_slice(&json).unwrap();
        assert_eq!(value["u"], "2026-03-04T05:06:07.123456Z");
        assert_eq!(value["i"], "00000000-0000-0000-0000-000000000007");
        assert_eq!(text.parse::<Cursor>(), Ok(cursor));
    }

    #[test]
    fn a_cursor_that_does_not_decode_is_request_malformed() {
        let not_json = URL_SAFE_NO_PAD.encode("nope");
        let no_id = URL_SAFE_NO_PAD.encode(r#"{"u":"2026-01-01T00:00:00Z"}"#);
        for text in ["", "!!!", not_json.as_str(), no_id.as_str()] {
            let error = text.parse::<Cursor>().unwrap_err();
            assert_eq!(error.code(), "request.malformed", "{text:?}");
            assert!(error.params().is_empty());
        }
    }

    #[test]
    fn a_page_request_bounds_its_limit() {
        let max = u16::try_from(PAGE_SIZE_MAX).unwrap();
        let default = u16::try_from(PAGE_SIZE_DEFAULT).unwrap();
        assert_eq!(PageRequest::new(None, None).limit, default);
        assert_eq!(PageRequest::new(None, Some(max + 1)).limit, max);
        assert_eq!(PageRequest::new(None, Some(0)).limit, PAGE_SIZE_MIN);
        assert_eq!(PageRequest::new(None, Some(7)).limit, 7);
    }

    #[test]
    fn pages_run_from_the_most_recent_without_gap_or_duplicate() {
        let items: Vec<Cursor> = (0..7)
            .map(|index| cursor(index / 2, index as u128))
            .collect();
        let mut request = PageRequest::new(None, Some(3));
        let mut seen = Vec::new();
        loop {
            let page = paginate(items.clone(), &request, |item| *item);
            seen.extend(page.items);
            let Some(next) = page.next_cursor else { break };
            request.cursor = Some(next);
        }
        let mut expected = items;
        expected.sort_by(|a, b| b.cmp(a));
        assert_eq!(seen, expected);
    }
}
