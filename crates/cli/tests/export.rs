//! `export`: writes an animation's files locally, refuses to overwrite without the flag, and
//! refuses a symbolic link — `docs/v1/mcp-cli.md`'s "A4 — CLI" tests.

#![allow(clippy::unwrap_used, reason = "a panic is a failed test")]

mod common;

use std::fs;

use common::{TestLibrary, life_pixel, scratch_dir, stderr, stdout};

#[tokio::test]
async fn export_writes_the_gif_into_the_working_directory_by_default() {
    let library = TestLibrary::seeded("Mascot").await;
    let out = scratch_dir();
    let id = library.animation.uuid().to_string();
    let library_path = library.path().display().to_string();
    let args = ["export", &id, "--format", "gif", "--library", &library_path];

    let output = life_pixel(out.path(), &args, &[]);

    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    assert!(text.starts_with("Exported 1 file(s)"), "{text}");
    let entries: Vec<_> = fs::read_dir(out.path()).unwrap().collect();
    assert_eq!(entries.len(), 1, "{text}");
}

#[tokio::test]
async fn a_second_export_needs_overwrite() {
    let library = TestLibrary::seeded("Mascot").await;
    let out = scratch_dir();
    let id = library.animation.uuid().to_string();
    let library_path = library.path().display().to_string();
    let args = ["export", &id, "--format", "gif", "--library", &library_path];
    life_pixel(out.path(), &args, &[]);

    let refused = life_pixel(out.path(), &args, &[]);

    let overwrite_args = [
        "export",
        &id,
        "--format",
        "gif",
        "--library",
        &library_path,
        "--overwrite",
    ];
    let replaced = life_pixel(out.path(), &overwrite_args, &[]);

    assert!(!refused.status.success());
    assert!(
        stderr(&refused).contains("already exists"),
        "{}",
        stderr(&refused)
    );
    assert!(replaced.status.success(), "{}", stderr(&replaced));
}

#[cfg(unix)]
#[tokio::test]
async fn a_symbolic_link_target_is_refused_even_with_overwrite() {
    let library = TestLibrary::seeded("Mascot").await;
    let out = scratch_dir();
    let elsewhere = scratch_dir();
    let target = elsewhere.path().join("target.gif");
    fs::write(&target, b"outside").unwrap();
    let id = library.animation.uuid().to_string();
    let library_path = library.path().display().to_string();
    let args = ["export", &id, "--format", "gif", "--library", &library_path];
    let first = life_pixel(out.path(), &args, &[]);
    let written = std::path::PathBuf::from(stdout(&first).lines().nth(1).unwrap());
    fs::remove_file(&written).unwrap();
    std::os::unix::fs::symlink(&target, &written).unwrap();

    let overwrite_args = [
        "export",
        &id,
        "--format",
        "gif",
        "--library",
        &library_path,
        "--overwrite",
    ];
    let refused = life_pixel(out.path(), &overwrite_args, &[]);

    assert!(!refused.status.success());
    assert!(
        stderr(&refused).contains("symbolic link"),
        "{}",
        stderr(&refused)
    );
    assert_eq!(fs::read(&target).unwrap(), b"outside");
}

#[tokio::test]
async fn an_unknown_animation_fails_with_its_message_on_stderr() {
    let library = TestLibrary::empty();
    let out = scratch_dir();
    let library_path = library.path().display().to_string();
    let args = [
        "export",
        "00000000-0000-0000-0000-000000000000",
        "--format",
        "gif",
        "--library",
        &library_path,
    ];

    let output = life_pixel(out.path(), &args, &[]);

    assert!(!output.status.success());
    assert_eq!(
        stderr(&output).trim(),
        "This animation does not exist, or was deleted."
    );
}
