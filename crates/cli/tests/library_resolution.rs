//! `--library` and `LIFE_PIXEL_LIBRARY`: which library `list` reads —
//! `docs/v1/mcp-cli.md`'s "A4 — CLI", "Library".

#![allow(clippy::unwrap_used, reason = "a panic is a failed test")]

mod common;

use common::{TestLibrary, life_pixel, stdout};

#[tokio::test]
async fn the_library_flag_is_used_over_the_environment_variable() {
    let flagged = TestLibrary::seeded("Flagged").await;
    let from_env = TestLibrary::seeded("FromEnv").await;
    let path = flagged.path().display().to_string();
    let env_path = from_env.path().display().to_string();

    let output = life_pixel(
        flagged.path(),
        &["list", "--library", &path],
        &[("LIFE_PIXEL_LIBRARY", &env_path)],
    );

    let text = stdout(&output);
    assert!(text.contains("Flagged"), "{text}");
    assert!(!text.contains("FromEnv"), "{text}");
}

#[tokio::test]
async fn the_environment_variable_is_used_without_the_flag() {
    let from_env = TestLibrary::seeded("FromEnv").await;
    let env_path = from_env.path().display().to_string();

    let output = life_pixel(
        from_env.path(),
        &["list"],
        &[("LIFE_PIXEL_LIBRARY", &env_path)],
    );

    assert!(stdout(&output).contains("FromEnv"), "{}", stdout(&output));
}
