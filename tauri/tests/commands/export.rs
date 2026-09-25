//! Saving the files of an export.

use std::fs;

use life_pixel_desktop::commands::export::{ExportFile, export_save_files};

use crate::harness::{FakeDialogs, Harness, base64, code};

fn file(name: &str, bytes: &[u8]) -> ExportFile {
    ExportFile {
        name: name.to_owned(),
        bytes: base64(bytes),
    }
}

#[tokio::test]
async fn one_file_is_saved_where_the_save_dialog_says() {
    let folder = tempfile::tempdir().unwrap();
    let target = folder.path().join("walk.gif");
    let dialogs = FakeDialogs {
        save_file: Some(target.clone()),
        ..FakeDialogs::default()
    };
    let harness = Harness::with(dialogs, |_| {});
    let saved = export_save_files(harness.state(), vec![file("walk.gif", b"GIF89a")]).await;
    assert_eq!(saved.unwrap(), Some(vec![target.display().to_string()]));
    assert_eq!(fs::read(&target).unwrap(), b"GIF89a");
}

#[tokio::test]
async fn several_files_go_into_the_folder_picked() {
    let folder = tempfile::tempdir().unwrap();
    let dialogs = FakeDialogs {
        folder: Some(folder.path().to_path_buf()),
        ..FakeDialogs::default()
    };
    let harness = Harness::with(dialogs, |_| {});
    let files = vec![file("walk-0.png", b"zero"), file("walk-1.png", b"one")];
    let saved = export_save_files(harness.state(), files)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved.len(), 2);
    assert_eq!(fs::read(folder.path().join("walk-1.png")).unwrap(), b"one");
}

#[tokio::test]
async fn a_cancelled_dialog_saves_nothing() {
    let harness = Harness::new();
    let one = export_save_files(harness.state(), vec![file("a.gif", b"a")]).await;
    assert_eq!(one.unwrap(), None);
    let several = export_save_files(harness.state(), vec![file("a", b"a"), file("b", b"b")]).await;
    assert_eq!(several.unwrap(), None);
}

#[tokio::test]
async fn the_export_folder_of_a_debug_build_needs_no_dialog() {
    let harness = Harness::with(FakeDialogs::default(), |options| {
        let folder = options
            .settings_file
            .parent()
            .unwrap()
            .with_file_name("exports");
        options.export_dir = Some(folder);
    });
    let saved = export_save_files(harness.state(), vec![file("walk.gif", b"GIF")]).await;
    let path = harness.folder.path().join("exports").join("walk.gif");
    assert_eq!(saved.unwrap(), Some(vec![path.display().to_string()]));
    assert_eq!(fs::read(path).unwrap(), b"GIF");
}

#[tokio::test]
async fn a_name_leaving_its_folder_or_bytes_not_in_base64_are_refused() {
    let harness = Harness::new();
    let escaping = export_save_files(harness.state(), vec![file("../walk.gif", b"x")]).await;
    assert_eq!(code(escaping), "request.malformed");
    let not_base64 = ExportFile {
        name: "walk.gif".into(),
        bytes: "%%".into(),
    };
    let result = export_save_files(harness.state(), vec![not_base64]).await;
    assert_eq!(code(result), "request.malformed");
    assert_eq!(
        code(export_save_files(harness.state(), Vec::new()).await),
        "request.malformed"
    );
}

#[tokio::test]
async fn a_file_that_cannot_be_written_is_export_write_failed() {
    let blocker = tempfile::NamedTempFile::new().unwrap();
    let dialogs = FakeDialogs {
        save_file: Some(blocker.path().join("walk.gif")),
        ..FakeDialogs::default()
    };
    let harness = Harness::with(dialogs, |_| {});
    let error = export_save_files(harness.state(), vec![file("walk.gif", b"x")])
        .await
        .unwrap_err();
    assert_eq!(error.code, "export.write_failed");
    assert!(error.params["path"].as_str().unwrap().ends_with("walk.gif"));
}
