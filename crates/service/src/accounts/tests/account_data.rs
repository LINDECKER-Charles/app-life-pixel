//! What its owner does with the account itself: exporting its data, and deleting it.

use std::sync::Arc;

use bytes::Bytes;
use zip::ZipArchive;

use super::{EMAIL, Harness, OTHER_PASSWORD, PASSWORD, START};
use crate::accounts::{ACCOUNT_FILE, AccountDeletion, AccountsError};
use crate::library::{Library, LibraryPorts};
use crate::local::LocalLibrary;
use crate::memory::{InMemoryLibraryStore, SequentialIds};
use crate::paging::PageRequest;
use crate::ports::LibraryStore;
use crate::ports::library_store::{AnimationFilter, AnimationRecord};
use crate::{AccountId, Owner, Plans};

/// A library over the harness's clock and events.
fn library(harness: &Harness) -> Library {
    let ports = LibraryPorts {
        store: Arc::new(InMemoryLibraryStore::new()),
        clock: harness.clock.clone(),
        ids: Arc::new(SequentialIds::new()),
        events: harness.events.clone(),
    };
    let plans = Plans {
        free_storage_bytes: u64::MAX,
        free_mcp_calls_per_day: 0,
    };
    Library::new(ports, plans)
}

/// A project of `account` holding the sample animation `title`.
async fn fill(library: &Library, account: AccountId, title: &str) {
    let owner = Owner::Account(account);
    let project = library.create_project(&owner, "Sprites").await.unwrap();
    let (_, document) = crate::testing::sample_document(title);
    let imported = library.import_animation(&owner, project.id, document);
    imported.await.unwrap();
}

/// The most recent animation of `account`, and its document.
async fn first_document(library: &Library, account: AccountId) -> (AnimationRecord, Bytes) {
    let owner = Owner::Account(account);
    let page = library.list_animations(&owner, AnimationFilter::default(), PageRequest::default());
    let id = page.await.unwrap().items[0].id;
    library.open_document(&owner, id).await.unwrap()
}

/// A temporary folder holding the entries of `file`.
fn unzipped(file: std::fs::File) -> tempfile::TempDir {
    let folder = tempfile::tempdir().unwrap();
    ZipArchive::new(file)
        .unwrap()
        .extract(folder.path())
        .unwrap();
    folder
}

#[tokio::test]
async fn an_export_unzipped_is_a_local_library_with_the_account_beside_it() {
    let harness = Harness::new();
    let library = library(&harness);
    let account = harness.sign_up().await.account.id;
    fill(&library, account, "Walk").await;
    let (hosted, hosted_document) = first_document(&library, account).await;

    let export = harness
        .accounts
        .export_data(&library, account)
        .await
        .unwrap();

    assert_eq!(export.file_name(), "life-pixel-export-2026-09-01.zip");
    assert_eq!(export.made_at, START);
    let folder = unzipped(export.file);
    let account_file = std::fs::read(folder.path().join(ACCOUNT_FILE)).unwrap();
    let account_file: serde_json::Value = serde_json::from_slice(&account_file).unwrap();
    let expected = serde_json::json!({
        "email": EMAIL, "language": "en", "plan": "free", "createdAt": "2026-09-01T12:00:00Z"
    });
    assert_eq!(account_file, expected);
    let local = LocalLibrary::open(folder.path()).unwrap();
    let projects = local.list_projects(&Owner::Local, PageRequest::default());
    let projects = projects.await.unwrap().items;
    let project = (projects[0].name.as_str(), projects[0].animation_count);
    assert_eq!((projects.len(), project), (1, ("Sprites", 1)));
    let (record, document) = local.read_document(&Owner::Local, hosted.id).await.unwrap();
    assert_eq!(record.project, hosted.project);
    assert_eq!(record.meta, hosted.meta);
    assert_eq!(document, hosted_document);
}

#[tokio::test]
async fn an_empty_library_exports_its_manifest_and_the_account() {
    let harness = Harness::new();
    let library = library(&harness);
    let account = harness.sign_up().await.account.id;

    let export = harness
        .accounts
        .export_data(&library, account)
        .await
        .unwrap();

    assert_eq!(export.bytes, export.file.metadata().unwrap().len());
    let zip = ZipArchive::new(export.file).unwrap();
    let names: Vec<_> = zip.file_names().collect();
    assert_eq!(names, ["library.json", ACCOUNT_FILE]);
}

#[tokio::test]
async fn deleting_asks_the_password_then_removes_the_library_and_the_account() {
    let harness = Harness::new();
    let library = library(&harness);
    let account = harness.sign_up().await.account.id;
    fill(&library, account, "Walk").await;
    let deletion = |password: &str| AccountDeletion {
        account,
        password: password.to_owned(),
    };

    let refused = harness
        .accounts
        .delete_with_library(&library, deletion(OTHER_PASSWORD))
        .await;
    assert_eq!(refused, Err(AccountsError::CurrentPassword));
    assert!(harness.accounts.account(account).await.is_ok());

    let deleted = harness
        .accounts
        .delete_with_library(&library, deletion(PASSWORD));
    deleted.await.unwrap();

    let owner = Owner::Account(account);
    let left = library.list_projects(&owner, PageRequest::default()).await;
    assert!(left.unwrap().items.is_empty());
    let gone = harness.accounts.account(account).await;
    assert_eq!(gone, Err(AccountsError::Unauthenticated));
    assert_eq!(harness.event_names().last(), Some(&"account_deleted"));
}
