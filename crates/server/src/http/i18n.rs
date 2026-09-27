//! The i18n endpoint: `/i18n/languages.json`, `/i18n/{code}.json` and the legal pages of
//! `/i18n/legal/{code}/{page}.md`, read from `LP_I18N_DIR` into memory at start, with a strong
//! `ETag` and `Cache-Control: no-cache`. A code, a page or a legal page absent from disk is
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

use super::problem::{Problem, codes};
use super::static_app::{CachedFile, NO_CACHE};
use crate::config::LegalIdentity;
use crate::state::AppState;

const LANGUAGES_FILE: &str = "languages.json";
const JSON: &str = "application/json";
const MARKDOWN: &str = "text/markdown; charset=utf-8";
/// Where the legal pages live under `LP_I18N_DIR`.
const LEGAL_DIR: &str = "legal";
/// The pages every language must have under `legal/<code>/`.
const LEGAL_PAGES: [&str; 3] = ["terms", "privacy", "notice"];

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
    /// A legal page is not valid UTF-8 text.
    #[error("{LEGAL_DIR}/{code}/{page}.md is not valid UTF-8")]
    LegalEncoding {
        /// The language code.
        code: String,
        /// The page: `terms`, `privacy` or `notice`.
        page: &'static str,
    },
}

/// `languages.json`, each catalogue it lists, and each language's legal pages, by file name.
#[derive(Debug, Default)]
pub struct Catalogues {
    files: HashMap<String, CachedFile>,
    legal: HashMap<(String, String), CachedFile>,
    languages: Vec<String>,
}

#[derive(Deserialize)]
struct Language {
    code: String,
}

impl Catalogues {
    /// Reads `languages.json` from `dir`, then the catalogue and the legal pages of each
    /// language it lists, filling `legal`'s placeholders from `identity`.
    ///
    /// # Errors
    ///
    /// When a file cannot be read, the list does not parse, a code is not a plain name, or a
    /// legal page is not valid UTF-8.
    pub fn load(dir: &Path, identity: &LegalIdentity) -> Result<Self, CatalogueError> {
        let list = read(dir, LANGUAGES_FILE)?;
        let languages = codes(&list)?;
        let mut files = HashMap::new();
        let mut legal = HashMap::new();
        for code in &languages {
            let name = format!("{code}.json");
            files.insert(name.clone(), catalogue_file(read(dir, &name)?));
            for page in LEGAL_PAGES {
                let body = read(dir, &format!("{LEGAL_DIR}/{code}/{page}.md"))?;
                let text = String::from_utf8(body.to_vec()).map_err(|_| {
                    CatalogueError::LegalEncoding {
                        code: code.clone(),
                        page,
                    }
                })?;
                let filled = fill_placeholders(&text, identity);
                legal.insert((code.clone(), page.to_owned()), legal_page_file(filled));
            }
        }
        files.insert(LANGUAGES_FILE.to_owned(), catalogue_file(list));
        Ok(Self {
            files,
            legal,
            languages,
        })
    }

    /// The codes `languages.json` lists, in its order: those an account may choose.
    #[must_use]
    pub fn languages(&self) -> &[String] {
        &self.languages
    }

    /// The file `name`: `languages.json` or `<code>.json`.
    #[must_use]
    pub fn file(&self, name: &str) -> Option<&CachedFile> {
        self.files.get(name)
    }

    /// The legal page `page` (`terms`, `privacy` or `notice`) of language `code`, its
    /// placeholders already filled from the configuration.
    #[must_use]
    pub fn legal_page(&self, code: &str, page: &str) -> Option<&CachedFile> {
        self.legal.get(&(code.to_owned(), page.to_owned()))
    }
}

/// The routes under `/i18n`, one line per route.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/{file}", get(catalogue))
        .route("/legal/{code}/{file}", get(legal_page))
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

async fn legal_page(
    State(state): State<AppState>,
    UrlPath((code, file)): UrlPath<(String, String)>,
    request: Request,
) -> Response {
    let page = known_page(&file);
    match page.and_then(|page| state.catalogues.legal_page(&code, page)) {
        Some(file) => file.respond(request.method(), request.headers()),
        None => Problem::new(codes::REQUEST_NOT_FOUND).into_response(),
    }
}

/// `file`, stripped of its `.md` suffix, when the result is one of `LEGAL_PAGES`.
fn known_page(file: &str) -> Option<&str> {
    file.strip_suffix(".md")
        .filter(|page| LEGAL_PAGES.contains(page))
}

/// The codes `list` — `languages.json` — lists, in its order, when each is a plain name.
fn codes(list: &[u8]) -> Result<Vec<String>, CatalogueError> {
    let languages: Vec<Language> =
        serde_json::from_slice(list).map_err(|_| CatalogueError::Languages)?;
    let codes = languages.into_iter().map(|Language { code }| code);
    codes
        .map(|code| {
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

fn legal_page_file(text: String) -> CachedFile {
    CachedFile::new(Bytes::from(text), MARKDOWN, NO_CACHE)
}

/// Replaces `{{publisher}}`, `{{address}}`, `{{contact}}`, `{{director}}` and `{{host}}` with
/// `identity`'s fields, so a self-hosted server shows its own identity.
fn fill_placeholders(text: &str, identity: &LegalIdentity) -> String {
    text.replace("{{publisher}}", &identity.publisher)
        .replace("{{address}}", &identity.address)
        .replace("{{contact}}", &identity.contact)
        .replace("{{director}}", &identity.director)
        .replace("{{host}}", &identity.host)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use axum::http::{HeaderMap, HeaderValue, Method, header};
    use tempfile::TempDir;

    use super::*;

    fn identity() -> LegalIdentity {
        LegalIdentity {
            publisher: "Ada's Studio".to_owned(),
            address: "1 Rue de Rivoli, Paris".to_owned(),
            contact: "legal@example.org".to_owned(),
            director: "Ada Lovelace".to_owned(),
            host: "Local host".to_owned(),
        }
    }

    /// `en` and `fr`, each with a catalogue and the three legal pages, `terms.md` carrying every
    /// placeholder.
    fn catalogues_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join(LANGUAGES_FILE),
            r#"[{"code":"en","name":"English"},{"code":"fr","name":"Français"}]"#,
        )
        .unwrap();
        for code in ["en", "fr"] {
            fs::write(dir.path().join(format!("{code}.json")), "{}").unwrap();
            let legal_dir = dir.path().join(LEGAL_DIR).join(code);
            fs::create_dir_all(&legal_dir).unwrap();
            fs::write(
                legal_dir.join("terms.md"),
                "# Terms\n\n{{publisher}}, {{address}}, {{contact}}, {{director}}, {{host}}.\n",
            )
            .unwrap();
            fs::write(
                legal_dir.join("privacy.md"),
                "# Privacy\n\n{{publisher}}.\n",
            )
            .unwrap();
            fs::write(legal_dir.join("notice.md"), "# Notice\n\n{{director}}.\n").unwrap();
        }
        dir
    }

    #[test]
    fn a_legal_page_is_filled_from_the_configuration() {
        let dir = catalogues_dir();
        let catalogues = Catalogues::load(dir.path(), &identity()).unwrap();

        let page = catalogues.legal_page("en", "terms").unwrap();
        let response = page.respond(&Method::GET, &HeaderMap::new());
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE),
            Some(&HeaderValue::from_static(MARKDOWN)),
        );
        assert!(response.headers().get(header::ETAG).is_some());
    }

    #[test]
    fn placeholders_are_replaced_by_their_configured_value() {
        let identity = identity();
        let filled = fill_placeholders(
            "{{publisher}} — {{address}} — {{contact}} — {{director}} — {{host}}",
            &identity,
        );
        assert_eq!(
            filled,
            "Ada's Studio — 1 Rue de Rivoli, Paris — legal@example.org — Ada Lovelace — Local host",
        );
    }

    #[test]
    fn known_page_accepts_only_a_listed_page_named_with_its_md_extension() {
        assert_eq!(known_page("terms.md"), Some("terms"));
        assert_eq!(known_page("privacy.md"), Some("privacy"));
        assert_eq!(known_page("notice.md"), Some("notice"));
        assert_eq!(known_page("terms.json"), None);
        assert_eq!(known_page("cookies.md"), None);
    }

    #[test]
    fn an_unknown_code_or_page_is_absent() {
        let dir = catalogues_dir();
        let catalogues = Catalogues::load(dir.path(), &identity()).unwrap();

        assert!(catalogues.legal_page("de", "terms").is_none());
        assert!(catalogues.legal_page("en", "cookies").is_none());
    }

    #[test]
    fn a_missing_legal_page_stops_loading_with_its_name() {
        let dir = catalogues_dir();
        fs::remove_file(dir.path().join(LEGAL_DIR).join("fr").join("notice.md")).unwrap();

        let error = Catalogues::load(dir.path(), &identity()).unwrap_err();
        assert!(
            matches!(error, CatalogueError::Read { file, .. } if file.contains("fr/notice.md"))
        );
    }
}
