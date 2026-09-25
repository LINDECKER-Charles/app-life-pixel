//! `cargo xtask measure-sizes`: S1's size checkpoint
//! ([docs/v1/format-player.md](../../../../docs/v1/format-player.md#s1--size-checkpoint)).
//!
//! Exports every sample of `samples/` as WASM, GIF and APNG through the compiler, and as a
//! lossless animated WebP through `img2webp` — measurement only —, measures each raw, gzipped
//! and with brotli, alongside the player alone and the loader, writes the table into
//! docs/export.md, and builds the demo of `target/demo/`.

use std::path::Path;

use anyhow::{Context, Result};
use cargo_metadata::MetadataCommand;
use life_pixel_compiler::{LOADER_JS, PLAYER_WASM};

mod artifacts;
mod compression;
mod demo;
mod samples;
mod table;
mod webp;

use compression::Sizes;
use table::Row;

/// Measures the size checkpoint's artefacts and writes their table and demo.
#[derive(clap::Args)]
pub struct MeasureSizes {}

/// A sample's WASM bundle, for the demo.
struct Export {
    /// The sample's file stem, e.g. `mascot-wave`.
    name: &'static str,
    /// The self-contained WebAssembly bundle.
    wasm: Vec<u8>,
}

impl MeasureSizes {
    /// Runs every step and reports the sizes it measured.
    pub fn run(self) -> Result<()> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("running cargo metadata")?;
        let root = metadata.workspace_root.as_std_path();
        let target_dir = metadata.target_directory.as_std_path();

        let (mut rows, exports) = measure_samples(root)?;
        rows.push(measured_row("player.wasm (no payload)", PLAYER_WASM)?);
        rows.push(measured_row("life-pixel.js (loader)", LOADER_JS.as_bytes())?);
        report(&rows);

        table::write(&root.join("docs/export.md"), &rows).context("writing docs/export.md")?;
        build_demo(root, target_dir, &exports)?;
        println!("measure-sizes: wrote docs/export.md and target/demo/");
        Ok(())
    }
}

/// Exports and measures every sample, returning their rows and their WASM bundles for the demo.
fn measure_samples(root: &Path) -> Result<(Vec<Row>, Vec<Export>)> {
    let mut rows = Vec::new();
    let mut exports = Vec::new();
    for name in samples::SAMPLE_NAMES {
        let animation =
            samples::load(root, name).with_context(|| format!("loading the sample {name}"))?;
        let files = artifacts::export_all(&animation)
            .with_context(|| format!("exporting the sample {name}"))?;
        let webp = webp::lossless_animated_webp(&files.png_frames_zip, &animation)
            .with_context(|| format!("encoding {name} as webp"))?;
        rows.push(measured_row(&format!("{name}.wasm"), &files.wasm)?);
        rows.push(measured_row(&format!("{name}.gif"), &files.gif)?);
        rows.push(measured_row(&format!("{name}.apng"), &files.apng)?);
        rows.push(measured_row(&format!("{name}.webp"), &webp)?);
        exports.push(Export {
            name,
            wasm: files.wasm,
        });
    }
    Ok((rows, exports))
}

fn report(rows: &[Row]) {
    for row in rows {
        println!(
            "measure-sizes: {}: raw {} B, gzip {} B, brotli {} B",
            row.name, row.sizes.raw, row.sizes.gzip, row.sizes.brotli
        );
    }
}

fn measured_row(name: &str, bytes: &[u8]) -> Result<Row> {
    Ok(Row {
        name: name.to_owned(),
        sizes: Sizes::measure(bytes).with_context(|| format!("measuring {name}"))?,
    })
}

fn build_demo(root: &Path, target_dir: &Path, exports: &[Export]) -> Result<()> {
    let borrowed: Vec<(&str, &[u8])> = exports
        .iter()
        .map(|export| (export.name, export.wasm.as_slice()))
        .collect();
    demo::build(root, target_dir, &borrowed).context("building target/demo/")
}
