//! The media type of a file of the built console, from its extension.

/// Extension to media type, for what an Angular build contains.
const CONTENT_TYPES: &[(&str, &str)] = &[
    ("css", "text/css; charset=utf-8"),
    ("gif", "image/gif"),
    ("html", "text/html; charset=utf-8"),
    ("ico", "image/x-icon"),
    ("jpg", "image/jpeg"),
    ("js", "text/javascript; charset=utf-8"),
    ("json", "application/json"),
    ("map", "application/json"),
    ("mjs", "text/javascript; charset=utf-8"),
    ("png", "image/png"),
    ("svg", "image/svg+xml"),
    ("txt", "text/plain; charset=utf-8"),
    ("webmanifest", "application/manifest+json"),
    ("webp", "image/webp"),
    ("woff", "font/woff"),
    ("woff2", "font/woff2"),
];

/// The type of anything else: never sniffed, thanks to `nosniff`.
const UNKNOWN: &str = "application/octet-stream";

/// The media type of the file at `path`.
#[must_use]
pub fn content_type(path: &str) -> &'static str {
    let extension = path.rsplit_once('.').map_or("", |(_, extension)| extension);
    CONTENT_TYPES
        .iter()
        .find(|(known, _)| known.eq_ignore_ascii_case(extension))
        .map_or(UNKNOWN, |(_, content_type)| content_type)
}
