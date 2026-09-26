//! French and English: `LC_ALL`/`LC_MESSAGES`/`LANG` pick the language of the CLI's own output —
//! `docs/v1/mcp-cli.md`'s "A4 — CLI" tests.

#![allow(clippy::unwrap_used, reason = "a panic is a failed test")]

mod common;

use common::{TestLibrary, life_pixel, scratch_dir, stderr};

#[tokio::test]
async fn an_unknown_animation_fails_in_english_by_default() {
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

#[tokio::test]
async fn an_unknown_animation_fails_in_french_with_lang_fr() {
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

    let output = life_pixel(out.path(), &args, &[("LANG", "fr_FR.UTF-8")]);

    assert!(!output.status.success());
    assert_eq!(
        stderr(&output).trim(),
        "Cette animation n\u{2019}existe pas ou a \u{e9}t\u{e9} supprim\u{e9}e."
    );
}

#[tokio::test]
async fn lc_all_wins_over_lang() {
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

    let output = life_pixel(
        out.path(),
        &args,
        &[("LC_ALL", "en_US.UTF-8"), ("LANG", "fr_FR.UTF-8")],
    );

    assert!(!output.status.success());
    assert_eq!(
        stderr(&output).trim(),
        "This animation does not exist, or was deleted."
    );
}
