//! The i18n endpoint: `/i18n/languages.json` and `/i18n/{code}.json`, read from `LP_I18N_DIR`
//! into memory at start, with a strong `ETag` and `Cache-Control: no-cache`. A code absent from
//! `languages.json` is `request.not_found`; nothing outside the folder is ever read.

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

use super::problem::{Problem, codes};
use super::static_app::{CachedFile, NO_CACHE};
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
        let languages: Vec<Language> =
            serde_json::from_slice(&list).map_err(|_| CatalogueError::Languages)?;
        let mut files = HashMap::new();
        for Language { code } in languages {
            let is_plain = !code.is_empty()
                && code
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
            if !is_plain {
                return Err(CatalogueError::Code { code });
            }
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

/// The routes under `/i18n`, one line per route.
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
