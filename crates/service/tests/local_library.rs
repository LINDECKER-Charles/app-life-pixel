//! The local library: the library store's contract on a temporary folder, then what only files
//! do — `library.json`, versions from the file's bytes, changes made behind the library's back,
//! an interrupted write, and files it does not know or cannot read.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

use std::fs;
use std::path::{Path, PathBuf};

use life_pixel_service::local::{LocalLibrary, LocalLibraryError};
use life_pixel_service::ports::LibraryStore;
use life_pixel_service::ports::library_store::{
    AnimationFilter, AnimationRecord, DocumentWrite, StoreError,
};
use life_pixel_service::testing::{
    StoreFixture, library_store_contract, new_animation, new_project, sample_document,
};
use life_pixel_service::{AccountId, Coded, Owner, PageRequest};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;
use time::OffsetDateTime;
use uuid::Uuid;

const OWNER: Owner = Owner::Local;

async fn local_library() -> StoreFixture {
    let folder = TempDir::new().unwrap();
    let library = LocalLibrary::open(folder.path()).unwrap();
    let account = Owner::Account(AccountId::from_uuid(Uuid::now_v7()));
    StoreFixture::new(library)
        .with_owners(Owner::Local, account)
        .with_guard(folder)
}

library_store_contract!(local_library);

/// A library on a new temporary folder, with one project holding the animation `title`.
async fn library_with(title: &str) -> (TempDir, LocalLibrary, AnimationRecord) {
    let folder = TempDir::new().unwrap();
    let library = LocalLibrary::open(folder.path()).unwrap();
    let project = new_project("Pets");
    library
        .create_project(&OWNER, project.clone())
        .await
        .unwrap();
    let new = new_animation(project.id, title);
    let created = library.create_animation(&OWNER, new, None).await.unwrap();
    (folder, library, created)
}

/// Where the layout puts the document of `animation`.
fn document_path(root: &Path, animation: &AnimationRecord) -> PathBuf {
    let project = root.join("projects").join(animation.project.to_string());
    project
        .join("animations")
        .join(format!("{}.json", animation.id))
}

/// The version of a file holding `bytes`: the first 6 bytes of their SHA-256.
fn version_of(bytes: &[u8]) -> u64 {
    let digest = Sha256::digest(bytes);
    digest[..6]
        .iter()
        .fold(0, |version, byte| version << 8 | u64::from(*byte))
}

fn write_of(animation: &AnimationRecord, title: &str) -> DocumentWrite {
    let (meta, document) = sample_document(title);
    DocumentWrite {
        id: animation.id,
        expected_version: animation.version,
        meta,
        document,
        at: OffsetDateTime::now_utc(),
    }
}

async fn listed(library: &LocalLibrary) -> Vec<AnimationRecord> {
    let filter = AnimationFilter::default();
    let page = library.list_animations(&OWNER, filter, PageRequest::default());
    page.await.unwrap().items
}

#[test]
fn opening_a_folder_writes_its_library_json_once() {
    let folder = TempDir::new().unwrap();
    LocalLibrary::open(folder.path()).unwrap();
    let path = folder.path().join("library.json");
    let written = fs::read(&path).unwrap();
    let manifest: Value = serde_json::from_slice(&written).unwrap();
    assert_eq!(
        manifest,
        json!({"format": "life-pixel/library", "version": 1})
    );

    LocalLibrary::open(folder.path()).unwrap();

    assert_eq!(fs::read(&path).unwrap(), written);
}

#[test]
fn a_later_library_version_is_unsupported() {
    let folder = TempDir::new().unwrap();
    let manifest = json!({"format": "life-pixel/library", "version": 2});
    fs::write(folder.path().join("library.json"), manifest.to_string()).unwrap();

    let error = LocalLibrary::open(folder.path()).err().unwrap();

    assert_eq!(error, LocalLibraryError::UnsupportedVersion { version: 2 });
    assert_eq!(error.code(), "library.unsupported_version");
    assert_eq!(error.params()["version"], 2);
}

#[test]
fn a_missing_folder_is_unavailable_until_created() {
    let folder = TempDir::new().unwrap();
    let missing = folder.path().join("Life Pixel");

    let error = LocalLibrary::open(&missing).err().unwrap();

    assert_eq!(error.code(), "library.unavailable");
    let path = missing.display().to_string();
    assert_eq!(error.params()["path"], Value::from(path));
    let library = LocalLibrary::create(&missing).unwrap();
    assert_eq!(library.path(), missing);
    assert!(missing.join("library.json").is_file());
}

#[test]
fn a_folder_whose_library_json_is_not_a_library_is_unavailable() {
    for manifest in ["not json", r#"{"format": "someone-else", "version": 1}"#] {
        let folder = TempDir::new().unwrap();
        fs::write(folder.path().join("library.json"), manifest).unwrap();

        let error = LocalLibrary::open(folder.path()).err().unwrap();

        assert_eq!(error.code(), "library.unavailable", "{manifest}");
    }
}

#[cfg(unix)]
#[test]
fn an_unwritable_folder_is_unavailable() {
    use std::os::unix::fs::PermissionsExt as _;
    let folder = TempDir::new().unwrap();
    let read_only = fs::Permissions::from_mode(0o555);
    fs::set_permissions(folder.path(), read_only).unwrap();

    let opened = LocalLibrary::open(folder.path());

    fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(opened.err().unwrap().code(), "library.unavailable");
}

#[tokio::test]
async fn a_version_is_the_start_of_the_sha256_of_the_file() {
    let (folder, library, created) = library_with("Cat").await;

    let bytes = fs::read(document_path(folder.path(), &created)).unwrap();

    assert_eq!(created.version, version_of(&bytes));
    assert!(created.version < 1 << 53);
    let written = library.write_document(&OWNER, write_of(&created, "Dog"), None);
    let written = written.await.unwrap();
    let bytes = fs::read(document_path(folder.path(), &written)).unwrap();
    assert_eq!(written.version, version_of(&bytes));
}

#[tokio::test]
async fn a_file_changed_behind_the_library_is_a_conflict() {
    let (folder, library, created) = library_with("Cat").await;
    let path = document_path(folder.path(), &created);
    let (_, changed) = sample_document("A dog drawn in another app");
    fs::write(&path, &changed).unwrap();

    let refused = library.write_document(&OWNER, write_of(&created, "Black cat"), None);

    let current = version_of(&changed);
    assert_eq!(refused.await, Err(StoreError::VersionConflict { current }));
    assert_eq!(fs::read(&path).unwrap(), changed);
    let read = library.get_animation(&OWNER, created.id).await.unwrap();
    assert_eq!(read.meta.title.as_str(), "A dog drawn in another app");
    assert_eq!(read.version, current);
}

#[tokio::test]
async fn a_crash_before_the_rename_leaves_the_old_document_whole() {
    let (folder, library, created) = library_with("Cat").await;
    let path = document_path(folder.path(), &created);
    let before = fs::read(&path).unwrap();
    let (_, next) = sample_document("Black cat");
    let temporary = path.with_file_name(".writing-interrupted");
    fs::write(&temporary, &next[..next.len() / 2]).unwrap();

    let (record, bytes) = library.read_document(&OWNER, created.id).await.unwrap();

    assert_eq!((record.version, &bytes[..]), (created.version, &before[..]));
    assert_eq!(listed(&library).await, std::slice::from_ref(&created));
    let written = library.write_document(&OWNER, write_of(&created, "Black cat"), None);
    assert!(written.await.is_ok());
}

#[tokio::test]
async fn files_the_library_does_not_know_are_ignored() {
    let (folder, library, created) = library_with("Cat").await;
    let root = folder.path();
    let document = fs::read(document_path(root, &created)).unwrap();
    let path = document_path(root, &created);
    let animations = path.parent().unwrap();
    let stray_project = root.join("projects").join(Uuid::now_v7().to_string());
    fs::create_dir_all(stray_project.join("animations")).unwrap();
    let stray_document = format!("animations/{}.json", Uuid::now_v7());
    fs::write(stray_project.join(stray_document), &document).unwrap();
    fs::create_dir_all(root.join("projects/not-an-id")).unwrap();
    fs::write(root.join("notes.txt"), "notes").unwrap();
    fs::write(root.join("projects/readme.md"), "readme").unwrap();
    fs::write(animations.join("cover.png"), "png").unwrap();
    fs::write(
        animations.join(format!("{}.txt", Uuid::now_v7())),
        &document,
    )
    .unwrap();

    let projects = library.list_projects(&OWNER, PageRequest::default()).await;

    let projects = projects.unwrap().items;
    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].animation_count, 1);
    assert_eq!(listed(&library).await, std::slice::from_ref(&created));
    assert_eq!(library.usage(&OWNER).await, Ok(created.document_bytes));
}

#[tokio::test]
async fn a_document_that_does_not_parse_is_left_out() {
    let (folder, library, created) = library_with("Cat").await;
    let broken = Uuid::now_v7();
    let path = document_path(folder.path(), &created);
    fs::write(path.with_file_name(format!("{broken}.json")), "{\"title\":").unwrap();

    assert_eq!(listed(&library).await, std::slice::from_ref(&created));
    let read = library.get_animation(&OWNER, broken.into()).await;
    assert!(read.is_err());
    assert_eq!(library.usage(&OWNER).await, Ok(created.document_bytes));
}

#[cfg(unix)]
#[tokio::test]
async fn unreadable_files_are_left_out() {
    use std::os::unix::fs::PermissionsExt as _;
    let (folder, library, hidden) = library_with("Cat").await;
    let kept = new_animation(hidden.project, "Dog");
    let kept = library.create_animation(&OWNER, kept, None).await.unwrap();
    let unreadable_project = new_project("Locked");
    library
        .create_project(&OWNER, unreadable_project.clone())
        .await
        .unwrap();
    let project_file = folder
        .path()
        .join(format!("projects/{}/project.json", unreadable_project.id));
    let no_access = fs::Permissions::from_mode(0o000);
    fs::set_permissions(document_path(folder.path(), &hidden), no_access.clone()).unwrap();
    fs::set_permissions(&project_file, no_access).unwrap();

    let projects = library.list_projects(&OWNER, PageRequest::default()).await;

    let ids: Vec<_> = projects.unwrap().items.iter().map(|p| p.id).collect();
    assert_eq!(ids, [hidden.project]);
    assert_eq!(listed(&library).await, [kept]);
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn a_write_keeps_the_creation_time() {
    let (_folder, library, created) = library_with("Cat").await;
    std::thread::sleep(std::time::Duration::from_millis(20));

    let written = library.write_document(&OWNER, write_of(&created, "Dog"), None);

    let written = written.await.unwrap();
    assert_eq!(written.created_at, created.created_at);
    assert!(written.updated_at > created.updated_at);
}

#[tokio::test]
async fn two_libraries_on_one_folder_let_exactly_one_write_win() {
    let (folder, first, created) = library_with("Cat").await;
    let second = LocalLibrary::open(folder.path()).unwrap();
    assert_eq!(
        second.get_animation(&OWNER, created.id).await,
        Ok(created.clone())
    );

    let writes = [(first, "Black cat"), (second, "White cat")].map(|(library, title)| {
        let write = write_of(&created, title);
        tokio::spawn(async move { library.write_document(&OWNER, write, None).await })
    });

    let mut won = 0;
    for write in writes {
        won += usize::from(write.await.unwrap().is_ok());
    }
    assert_eq!(won, 1);
}
