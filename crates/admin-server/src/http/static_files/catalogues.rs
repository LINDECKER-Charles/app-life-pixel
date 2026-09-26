//! The catalogues the console reads: `/i18n/languages.json` and `/i18n/{code}.json`, read from
//! `LPA_I18N_DIR` — the repository's `i18n/`, as the server's — into memory at start, with a
//! strong `ETag` and `Cache-Control: no-cache`. A file absent from the list is
//! `request.not_found`; nothing outside the folder is ever read.

use std::collections::HashMap;
use std::io;
use std::path::Path;

use axum::Router;
use axum::extract::{Path as UrlPath, Request, State};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use bytes::Bytes;
use serde::Deserialize;
use thiserror::Error;

use super::{CachedFile, NO_CACHE};
use crate::http::problem::{Problem, codes};
use crate::state::AppState;

const LANGUAGES_FILE: &str = "languages.json";
const JSON: &str = "application/json";

/// Why the catalogues could not be read at start.
#[derive(Debug, Error)]
pub enum CatalogueError {
    /// A file of the folder could not be read.
    #[error("cannot read {file}: {source}")]
    Read {
        /// The file's name.
        file: String,
        /// The failure.
        source: io::Error,
    },
    /// `languages.json` is not a list of languages.
    #[error("{LANGUAGES_FILE} is not a list of languages, each with a code")]
    Languages,
    /// A code that could name a file outside the folder.
    #[error("the language code {code:?} is not lowercase letters, digits and hyphens")]
    Code {
        /// The code.
        code: String,
    },
}

/// `languages.json` and each catalogue it lists, by file name.
#[derive(Debug, Default)]
pub struct Catalogues {
    files: HashMap<String, CachedFile>,
}

#[derive(Deserialize)]
struct Language {
    code: String,
}

impl Catalogues {
    /// Reads `languages.json` from `dir`, then the catalogue of each language it lists.
    ///
    /// # Errors
    ///
    /// When a file cannot be read, the list does not parse, or a code is not a plain name.
    pub fn load(dir: &Path) -> Result<Self, CatalogueError> {
        let list = read(dir, LANGUAGES_FILE)?;
        let mut files = HashMap::new();
        for code in language_codes(&list)? {
            let name = format!("{code}.json");
            files.insert(name.clone(), catalogue_file(read(dir, &name)?));
        }
        files.insert(LANGUAGES_FILE.to_owned(), catalogue_file(list));
        Ok(Self { files })
    }

    /// The file `name`: `languages.json` or `<code>.json`.
    #[must_use]
    pub fn file(&self, name: &str) -> Option<&CachedFile> {
        self.files.get(name)
    }
}

/// The routes under `/i18n`.
pub fn router() -> Router<AppState> {
    Router::new().route("/{file}", get(catalogue))
}

async fn catalogue(
    State(state): State<AppState>,
    UrlPath(file): UrlPath<String>,
    request: Request,
) -> Response {
    match state.catalogues.file(&file) {
        Some(file) => file.respond(request.method(), request.headers()),
        None => Problem::new(codes::REQUEST_NOT_FOUND).into_response(),
    }
}

/// The codes `list` — `languages.json` — lists, in its order, when each is a plain name.
fn language_codes(list: &[u8]) -> Result<Vec<String>, CatalogueError> {
    let languages: Vec<Language> =
        serde_json::from_slice(list).map_err(|_| CatalogueError::Languages)?;
    languages
        .into_iter()
        .map(|Language { code }| {
            let is_plain = !code.is_empty()
                && code
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
            if is_plain {
                Ok(code)
            } else {
                Err(CatalogueError::Code { code })
            }
        })
        .collect()
}

fn read(dir: &Path, name: &str) -> Result<Bytes, CatalogueError> {
    std::fs::read(dir.join(name))
        .map(Bytes::from)
        .map_err(|source| CatalogueError::Read {
            file: name.to_owned(),
            source,
        })
}

fn catalogue_file(body: Bytes) -> CachedFile {
    CachedFile::new(body, JSON, NO_CACHE)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn repository_catalogues() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../i18n")
    }

    #[test]
    fn the_repository_catalogues_load() {
        let catalogues = Catalogues::load(&repository_catalogues()).unwrap();
        assert!(catalogues.file("languages.json").is_some());
        assert!(catalogues.file("en.json").is_some());
        assert!(catalogues.file("../Cargo.toml").is_none());
    }

    #[test]
    fn a_code_naming_another_folder_is_refused() {
        let error = language_codes(br#"[{"code":"../etc"}]"#).unwrap_err();
        assert!(matches!(error, CatalogueError::Code { .. }));
    }
}
