//! `/i18n`: the catalogues from `LP_I18N_DIR`, with a strong `ETag` and `304`, and nothing
//! outside the languages `languages.json` lists.

mod common;

use common::{TestServer, get_with};
use sha2::{Digest, Sha256};

fn catalogue_bytes(file: &str) -> Vec<u8> {
    let path = format!("{}/../../i18n/{file}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read(&path).unwrap_or_else(|error| panic!("{path}: {error}"))
}

#[tokio::test]
async fn a_catalogue_has_a_strong_etag_of_its_content() {
    let server = TestServer::new();
    for file in ["languages.json", "en.json", "fr.json"] {
        let answer = server.get(&format!("/i18n/{file}")).await;
        let content = catalogue_bytes(file);
        assert_eq!(answer.status, 200, "{file}");
        assert_eq!(answer.body, content, "{file}");
        let digest: String = Sha256::digest(&content)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        assert_eq!(answer.header("etag"), format!("\"{digest}\""));
        assert_eq!(answer.header("cache-control"), "no-cache");
        assert!(
            answer
                .header("content-type")
                .starts_with("application/json")
        );
    }
}

#[tokio::test]
async fn a_matching_etag_is_not_modified() {
    let server = TestServer::new();
    let etag = server.get("/i18n/fr.json").await.header("etag").to_owned();
    for if_none_match in [
        etag.clone(),
        format!("W/{etag}"),
        format!("\"other\", {etag}"),
        "*".to_owned(),
    ] {
        let answer = server
            .send(get_with("/i18n/fr.json", "if-none-match", &if_none_match))
            .await;
        assert_eq!(answer.status, 304);
        assert!(answer.body.is_empty());
    }
    let stale = get_with("/i18n/fr.json", "if-none-match", "\"stale\"");
    assert_eq!(server.send(stale).await.status, 200);
}

#[tokio::test]
async fn an_unlisted_language_or_file_is_not_found() {
    let server = TestServer::new();
    for path in [
        "/i18n/de.json",
        "/i18n/en",
        "/i18n/..%2FCargo.toml",
        "/i18n/",
        "/i18n",
    ] {
        server
            .get(path)
            .await
            .assert_problem(404, "request.not_found");
    }
}
