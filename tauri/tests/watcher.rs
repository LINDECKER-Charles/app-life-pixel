//! The library watcher: a file written by hand in a temporary library reaches its listener as a
//! change naming its ids, and the watcher follows the library to a new folder.

#![allow(clippy::unwrap_used)] // A helper fails its test by panicking, as the test would.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use life_pixel_desktop::dialogs::Dialogs;
use life_pixel_desktop::settings::Settings;
use life_pixel_desktop::state::{DesktopState, StateOptions};
use life_pixel_desktop::watcher::{LibraryChange, LibraryWatcher};
use life_pixel_service::local::LocalLibrary;
use life_pixel_service::testing::sample_document;

const PROJECT: &str = "0190a4b2-7c3d-7e4f-8a5b-6c7d8e9fa0b1";
const ANIMATION: &str = "0190a4b2-7c3d-7e4f-8a5b-6c7d8e9fa0b2";
/// Far longer than the 300 ms debounce, for a loaded machine's file-system events.
const PATIENCE: Duration = Duration::from_secs(20);

/// A watcher whose changes arrive on the returned channel.
fn watcher() -> (LibraryWatcher, Receiver<LibraryChange>) {
    let (sender, receiver) = channel();
    let watcher = LibraryWatcher::new(move |change| drop(sender.send(change)));
    (watcher, receiver)
}

/// Writes an animation's document by hand, as an agent's tool or a person would.
fn write_by_hand(library: &Path) {
    let folder = library.join("projects").join(PROJECT).join("animations");
    std::fs::create_dir_all(&folder).unwrap();
    let document = sample_document("By hand").1;
    std::fs::write(folder.join(format!("{ANIMATION}.json")), document).unwrap();
}

/// Waits for a change naming the animation, skipping those that do not (the folders created).
fn wait_for_animation(receiver: &Receiver<LibraryChange>) -> LibraryChange {
    let deadline = Instant::now() + PATIENCE;
    loop {
        let left = deadline.saturating_duration_since(Instant::now());
        let Ok(change) = receiver.recv_timeout(left) else {
            panic!("no change naming the animation");
        };
        if change.animation_ids.contains(ANIMATION) {
            return change;
        }
    }
}

#[test]
fn a_document_written_by_hand_is_reported_with_its_ids() {
    let library = tempfile::tempdir().unwrap();
    LocalLibrary::create(library.path()).unwrap();
    let (watcher, receiver) = watcher();
    watcher.watch(library.path());
    write_by_hand(library.path());
    let change = wait_for_animation(&receiver);
    assert_eq!(change.project_ids, BTreeSet::from([PROJECT.to_owned()]));
    assert_eq!(change.animation_ids, BTreeSet::from([ANIMATION.to_owned()]));
    let json = serde_json::to_value(&change).unwrap();
    assert_eq!(json["animationIds"][0], ANIMATION);
    assert_eq!(json["projectIds"][0], PROJECT);
}

struct NoDialogs;

#[async_trait]
impl Dialogs for NoDialogs {
    async fn save_file(&self, _file_name: &str) -> Option<std::path::PathBuf> {
        None
    }

    async fn pick_folder(&self) -> Option<std::path::PathBuf> {
        None
    }
}

#[tokio::test]
async fn the_watcher_follows_the_library_to_its_new_folder() {
    let folder = tempfile::tempdir().unwrap();
    let options = StateOptions {
        settings_file: folder.path().join("settings.json"),
        default_library: folder.path().join("Life Pixel"),
        library_override: None,
        export_dir: None,
        cli_path: None,
    };
    let state = DesktopState::open(options, Arc::new(NoDialogs));
    let (watcher, receiver) = watcher();
    state.watch_library(watcher).await;
    let other = folder.path().join("Other");
    LocalLibrary::create(&other).unwrap();
    let settings = Settings {
        library_path: Some(other.display().to_string()),
        ..Settings::default()
    };
    state.apply_settings(settings).await.unwrap();

    write_by_hand(&other);
    let change = tokio::task::spawn_blocking(move || wait_for_animation(&receiver));
    assert!(change.await.unwrap().project_ids.contains(PROJECT));
}
