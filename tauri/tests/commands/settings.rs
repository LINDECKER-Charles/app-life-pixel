//! The settings commands: the file written and read back, the library reopened.

use life_pixel_desktop::commands::library::{library_create_project, library_list_projects};
use life_pixel_desktop::commands::settings::*;
use life_pixel_desktop::settings::{Motion, Settings, Theme};

use crate::harness::{FakeDialogs, Harness, code};

/// The names of the projects of the library in use.
async fn project_names(harness: &Harness) -> Vec<String> {
    let page = library_list_projects(harness.state(), None, None)
        .await
        .unwrap();
    let json = serde_json::to_value(page).unwrap();
    let items = json["items"].as_array().unwrap();
    items
        .iter()
        .map(|item| item["name"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn the_settings_start_with_the_default_library() {
    let harness = Harness::new();
    let settings = settings_get(harness.state()).await.unwrap();
    let expected = Settings {
        library_path: Some(harness.default_library().display().to_string()),
        ..Settings::default()
    };
    assert_eq!(settings, expected);
    assert!(harness.default_library().join("library.json").is_file());
}

#[tokio::test]
async fn the_settings_are_written_then_read_back_after_a_restart() {
    let harness = Harness::new();
    let mut settings = settings_get(harness.state()).await.unwrap();
    settings.language = Some("fr".into());
    settings.theme = Theme::Dark;
    settings.motion = Motion::Reduce;
    let answered = settings_set(harness.state(), settings.clone())
        .await
        .unwrap();
    assert_eq!(answered, settings);
    let stored = Settings::read(&harness.settings_file());
    assert_eq!(
        stored.library_path, None,
        "the default folder is not written"
    );
    let harness = harness.restart();
    assert_eq!(settings_get(harness.state()).await.unwrap(), settings);
}

#[tokio::test]
async fn a_new_library_path_reopens_the_library_there() {
    let harness = Harness::new();
    library_create_project(harness.state(), "Default".into())
        .await
        .unwrap();
    let other = harness.new_folder("Other");
    let mut settings = settings_get(harness.state()).await.unwrap();
    settings.library_path = Some(other.display().to_string());
    settings_set(harness.state(), settings.clone())
        .await
        .unwrap();
    assert!(project_names(&harness).await.is_empty());
    library_create_project(harness.state(), "Other".into())
        .await
        .unwrap();

    let harness = harness.restart();
    assert_eq!(project_names(&harness).await, ["Other"]);
    settings.library_path = None;
    let back = settings_set(harness.state(), settings).await.unwrap();
    assert_eq!(
        back.library_path,
        Some(harness.default_library().display().to_string())
    );
    assert_eq!(project_names(&harness).await, ["Default"]);
}

#[tokio::test]
async fn a_folder_that_cannot_be_opened_changes_nothing() {
    let harness = Harness::new();
    let before = settings_get(harness.state()).await.unwrap();
    let mut settings = before.clone();
    settings.library_path = Some(harness.folder.path().join("missing").display().to_string());
    settings.theme = Theme::Light;
    let error = settings_set(harness.state(), settings).await.unwrap_err();
    assert_eq!(error.code, "library.unavailable");
    assert_eq!(settings_get(harness.state()).await.unwrap(), before);
    assert!(!harness.settings_file().exists());
}

#[tokio::test]
async fn a_library_that_cannot_be_opened_at_start_answers_why() {
    let harness = Harness::new();
    let gone = harness.new_folder("Gone");
    let settings = Settings {
        library_path: Some(gone.display().to_string()),
        ..Settings::default()
    };
    settings_set(harness.state(), settings).await.unwrap();
    std::fs::remove_dir_all(&gone).unwrap();
    let harness = harness.restart();
    let listed = library_list_projects(harness.state(), None, None).await;
    assert_eq!(code(listed), "library.unavailable");
    let info = settings_get(harness.state()).await.unwrap();
    assert_eq!(info.library_path, Some(gone.display().to_string()));
}

#[tokio::test]
async fn the_library_variable_wins_and_is_never_written() {
    let override_folder = tempfile::tempdir().unwrap();
    let path = override_folder.path().join("Library");
    let chosen = path.clone();
    let harness = Harness::with(FakeDialogs::default(), move |options| {
        options.library_override = Some(chosen);
    });
    let mut settings = settings_get(harness.state()).await.unwrap();
    assert_eq!(settings.library_path, Some(path.display().to_string()));
    assert!(path.join("library.json").is_file());
    settings.theme = Theme::Dark;
    settings_set(harness.state(), settings).await.unwrap();
    assert_eq!(Settings::read(&harness.settings_file()).library_path, None);
}

#[tokio::test]
async fn a_language_that_is_not_a_code_is_refused() {
    let harness = Harness::new();
    let settings = Settings {
        language: Some("../../etc".into()),
        ..Settings::default()
    };
    assert_eq!(
        code(settings_set(harness.state(), settings).await),
        "request.malformed"
    );
}

#[tokio::test]
async fn the_library_folder_is_picked_in_the_dialog() {
    let harness = Harness::new();
    assert_eq!(
        settings_pick_library_folder(harness.state()).await.unwrap(),
        None
    );
    let folder = harness.new_folder("Picked");
    let dialogs = FakeDialogs {
        folder: Some(folder.clone()),
        ..FakeDialogs::default()
    };
    let harness = Harness::with(dialogs, |_| {});
    let picked = settings_pick_library_folder(harness.state()).await.unwrap();
    assert!(picked.unwrap().ends_with("Picked"));
}
