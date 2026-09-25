//! The built app, read from `LP_APP_DIR` into memory at start: every `GET` that finds no file,
//! outside `/api`, `/i18n` and `/mcp`, gets `index.html`, for the app's own routes. Files
//! named with a content hash are cached for a year; everything else is revalidated.

mod cached_file;
mod content_type;

use std::collections::HashMap;
use std::io;
use std::path::Path;

use axum::extract::{Request, State};
use axum::http::header::ALLOW;
use axum::http::{HeaderValue, Method};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;

use super::problem::{Problem, codes};
use crate::state::AppState;

pub use cached_file::{CachedFile, NO_CACHE};
pub use content_type::content_type;

/// `Cache-Control` of a file named with its content hash: it never changes under that name.
pub const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// The first path segments the app never answers, even with `index.html`.
const RESERVED_SEGMENTS: [&str; 3] = ["api", "i18n", "mcp"];
const INDEX: &str = "index.html";
/// The engine's files keep their names from one build to the next: never immutable.
const ENGINE_FOLDER: &str = "engine/";
/// The length of the content hash the Angular build puts in its file names.
const CONTENT_HASH_CHARS: usize = 8;

/// The files of the built app, by path relative to `LP_APP_DIR`.
#[derive(Debug, Default)]
pub struct StaticApp {
    files: HashMap<String, CachedFile>,
}

impl StaticApp {
    /// Reads every file under `dir`, following no symbolic link. A missing folder gives an app
    /// without files — the API still serves —, and a warning.
    ///
    /// # Errors
    ///
    /// When a file or a folder under `dir` cannot be read.
    pub fn load(dir: &Path) -> io::Result<Self> {
        let mut app = Self::default();
        if !dir.is_dir() {
            tracing::warn!(dir = %dir.display(), "no built app: only the API is served");
            return Ok(app);
        }
        app.collect(dir, dir)?;
        Ok(app)
    }

    /// The file at `path`, relative to the app's folder.
    #[must_use]
    pub fn file(&self, path: &str) -> Option<&CachedFile> {
        self.files.get(path)
    }

    fn collect(&mut self, root: &Path, dir: &Path) -> io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                self.collect(root, &entry.path())?;
            } else if file_type.is_file() {
                self.add(root, &entry.path())?;
            }
        }
        Ok(())
    }

    fn add(&mut self, root: &Path, path: &Path) -> io::Result<()> {
        let Some(relative) = relative_path(root, path) else {
            return Ok(());
        };
        let body = Bytes::from(std::fs::read(path)?);
        let file = CachedFile::new(body, content_type(&relative), cache_control(&relative));
        self.files.insert(relative, file);
        Ok(())
    }
}

/// The fallback of the public router: the app's file, or `index.html`.
pub async fn serve(State(state): State<AppState>, request: Request) -> Response {
    let path = request.uri().path();
    if is_reserved(path) {
        return Problem::new(codes::REQUEST_NOT_FOUND).into_response();
    }
    let method = request.method();
    if method != Method::GET && method != Method::HEAD {
        let mut response = Problem::new(codes::REQUEST_METHOD_NOT_ALLOWED).into_response();
        response
            .headers_mut()
            .insert(ALLOW, HeaderValue::from_static("GET, HEAD"));
        return response;
    }
    let app = &state.static_app;
    let requested = path.trim_start_matches('/');
    let file = app.file(requested).or_else(|| app.file(INDEX));
    file.map_or_else(
        || Problem::new(codes::REQUEST_NOT_FOUND).into_response(),
        |file| file.respond(method, request.headers()),
    )
}

/// Whether `path` is `/api`, `/i18n` or `/mcp`, or under one of them.
fn is_reserved(path: &str) -> bool {
    let first = path.trim_start_matches('/').split('/').next();
    first.is_some_and(|segment| RESERVED_SEGMENTS.contains(&segment))
}

/// `a/b.js` for `<root>/a/b.js`; `None` for a name that is not UTF-8.
fn relative_path(root: &Path, path: &Path) -> Option<String> {
    let parts = path
        .strip_prefix(root)
        .ok()?
        .components()
        .map(|component| component.as_os_str().to_str())
        .collect::<Option<Vec<_>>>()?;
    Some(parts.join("/"))
}

fn cache_control(path: &str) -> &'static str {
    if is_content_hashed(path) {
        IMMUTABLE
    } else {
        NO_CACHE
    }
}

/// Whether `path` is a script or a style sheet named `<name>-XXXXXXXX.js` or `.css`, the
/// hash in capitals and digits, outside the engine's folder.
fn is_content_hashed(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let stem = name
        .strip_suffix(".js")
        .or_else(|| name.strip_suffix(".css"));
    let hash = stem
        .and_then(|stem| stem.rsplit_once('-'))
        .map(|(_, hash)| hash);
    let is_hash = |hash: &str| {
        hash.len() == CONTENT_HASH_CHARS
            && hash
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
    };
    !path.starts_with(ENGINE_FOLDER) && hash.is_some_and(is_hash)
}
