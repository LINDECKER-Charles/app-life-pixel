//! The measured sizes as a markdown table, spliced into docs/export.md between its markers.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use super::compression::Sizes;

/// Where the table starts in docs/export.md's "Measured sizes" section.
const START_MARKER: &str = "<!-- sizes:start -->";
/// Where it ends.
const END_MARKER: &str = "<!-- sizes:end -->";

/// One row of the table: the artefact's name and its three sizes.
pub struct Row {
    /// The artefact, e.g. `mascot-wave.wasm` or `life-pixel.js (loader)`.
    pub name: String,
    /// Its raw, gzip and brotli sizes.
    pub sizes: Sizes,
}

/// Replaces the table between [`START_MARKER`] and [`END_MARKER`] of `export_md` with `rows`.
pub fn write(export_md: &Path, rows: &[Row]) -> Result<()> {
    let content = fs::read_to_string(export_md)
        .with_context(|| format!("reading {}", export_md.display()))?;
    let start = content
        .find(START_MARKER)
        .with_context(|| format!("{} has no {START_MARKER}", export_md.display()))?;
    let end = content
        .find(END_MARKER)
        .with_context(|| format!("{} has no {END_MARKER}", export_md.display()))?;
    if end < start {
        bail!("{} has {END_MARKER} before {START_MARKER}", export_md.display());
    }
    let before = &content[..start + START_MARKER.len()];
    let after = &content[end..];
    let updated = format!("{before}\n{}\n{after}", render(rows));
    fs::write(export_md, updated).with_context(|| format!("writing {}", export_md.display()))
}

/// The table's markdown text, one row per artefact.
fn render(rows: &[Row]) -> String {
    let mut table = String::new();
    let _ = writeln!(table, "| Artefact | Raw | Gzip 9 | Brotli 11 |");
    let _ = writeln!(table, "|---|---|---|---|");
    for row in rows {
        let _ = writeln!(
            table,
            "| `{}` | {} B | {} B | {} B |",
            row.name, row.sizes.raw, row.sizes.gzip, row.sizes.brotli
        );
    }
    table.trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_replaces_only_what_is_between_the_markers() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.md");
        fs::write(&path, "before\n<!-- sizes:start -->\nold\n<!-- sizes:end -->\nafter\n").unwrap();
        let rows = [Row {
            name: "sample.wasm".to_owned(),
            sizes: Sizes {
                raw: 10,
                gzip: 8,
                brotli: 6,
            },
        }];
        write(&path, &rows).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(content.starts_with("before\n<!-- sizes:start -->\n"));
        assert!(content.contains("| `sample.wasm` | 10 B | 8 B | 6 B |"));
        assert!(content.ends_with("<!-- sizes:end -->\nafter\n"));
        assert!(!content.contains("old"));
    }

    #[test]
    fn writing_twice_gives_the_same_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("export.md");
        fs::write(&path, "<!-- sizes:start -->\n<!-- sizes:end -->\n").unwrap();
        let rows = [Row {
            name: "a".to_owned(),
            sizes: Sizes {
                raw: 1,
                gzip: 1,
                brotli: 1,
            },
        }];
        write(&path, &rows).unwrap();
        let first = fs::read_to_string(&path).unwrap();
        write(&path, &rows).unwrap();
        let second = fs::read_to_string(&path).unwrap();
        assert_eq!(first, second);
    }
}
