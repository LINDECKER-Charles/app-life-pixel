//! The four artefacts one sample is measured as: the compiler's WASM, GIF and APNG, and the PNG
//! frames a lossless animated WebP is built from — [`super::webp`], through `img2webp`.

use anyhow::{Context, Result};
use life_pixel_compiler::{ClassicOptions, export_apng, export_gif, export_png_frames, export_wasm};
use life_pixel_core::Animation;

/// What `cargo xtask measure-sizes` exports a sample as, before compression.
pub struct SampleExports {
    /// The self-contained WebAssembly bundle.
    pub wasm: Vec<u8>,
    /// The classic GIF.
    pub gif: Vec<u8>,
    /// The classic APNG.
    pub apng: Vec<u8>,
    /// A zip of one indexed PNG per frame, [`super::webp::lossless_animated_webp`]'s source.
    pub png_frames_zip: Vec<u8>,
}

/// Every frame, at the animation's own size and scale — the whole animation for a sample without
/// a tag, since `measure-sizes` reports one size per sample, not per tag.
fn export_options() -> ClassicOptions {
    ClassicOptions::default()
}

/// Exports `animation` the four ways a sample is measured, all at once so that a failure names
/// which one broke.
pub fn export_all(animation: &Animation) -> Result<SampleExports> {
    let options = export_options();
    let wasm = export_wasm(animation).context("exporting the wasm bundle")?;
    let gif = export_gif(animation, &options)
        .context("exporting the gif")?
        .bytes;
    let apng = export_apng(animation, &options)
        .context("exporting the apng")?
        .bytes;
    let png_frames_zip = export_png_frames(animation, &options)
        .context("exporting the png frames")?
        .bytes;
    Ok(SampleExports {
        wasm,
        gif,
        apng,
        png_frames_zip,
    })
}
