//! The four samples of `samples/`, loaded by name — the table of docs/v1/format-player.md's
//! "S1 — Size checkpoint".

use std::path::Path;

use anyhow::{Context, Result};
use life_pixel_core::Animation;
use life_pixel_core::serialize::read_document;

/// The samples' file stems, in the order their table lists them.
pub const SAMPLE_NAMES: [&str; 4] = ["mascot-wave", "loader-dots", "hero-run", "empty-state"];

/// Reads and validates `samples/<name>.json`, relative to the workspace `root`.
pub fn load(root: &Path, name: &str) -> Result<Animation> {
    let path = root.join("samples").join(format!("{name}.json"));
    let bytes = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    read_document(&bytes).with_context(|| format!("{} is not a valid document", path.display()))
}
