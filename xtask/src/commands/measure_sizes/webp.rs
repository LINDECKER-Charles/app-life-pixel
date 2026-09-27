//! A lossless animated WebP built from a sample's PNG frames through `img2webp` — measurement
//! only, docs/v1/format-player.md's S1: the product does not export WebP (docs/export.md's
//! "Classic formats" lists it "later").

use std::io::Cursor;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, ensure};
use life_pixel_core::Animation;
use tempfile::TempDir;
use zip::ZipArchive;

/// `img2webp`'s loop count: infinite, as the player loops a tag by default.
const INFINITE_LOOP: &str = "0";
/// `img2webp`'s compression effort, lossless mode's own scale (0 fastest, 6 slowest): the
/// slowest, since this WebP is for measurement, not for serving.
const COMPRESSION_METHOD: &str = "6";

/// Encodes `png_frames_zip` (one indexed PNG per frame, in play order, from
/// [`super::artifacts::export_all`]) as a lossless animated WebP, each frame kept at
/// `animation`'s own duration. Deterministic: `img2webp` is a pure encoder.
///
/// # Errors
///
/// The zip cannot be read, its frame count does not match `animation`'s, or `img2webp` is
/// missing or fails.
pub fn lossless_animated_webp(png_frames_zip: &[u8], animation: &Animation) -> Result<Vec<u8>> {
    let directory = TempDir::new().context("creating a temporary directory")?;
    let frame_paths = extract_frames(png_frames_zip, directory.path())?;
    let durations: Vec<u32> = animation
        .frames()
        .iter()
        .map(|frame| u32::from(frame.duration_ms()))
        .collect();
    ensure!(
        frame_paths.len() == durations.len(),
        "the png frames zip has {} frames, the animation has {}",
        frame_paths.len(),
        durations.len()
    );
    let output_path = directory.path().join("out.webp");
    run_img2webp(&frame_paths, &durations, &output_path)?;
    std::fs::read(&output_path).context("reading img2webp's output")
}

/// Writes every entry of `zip`, in order, as `frame-<index>.png` under `directory`.
fn extract_frames(zip: &[u8], directory: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut archive = ZipArchive::new(Cursor::new(zip)).context("reading the png frames zip")?;
    let mut paths = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .with_context(|| format!("reading frame {index} of the png frames zip"))?;
        let path = directory.join(format!("frame-{index:04}.png"));
        let mut file = std::fs::File::create(&path)
            .with_context(|| format!("creating {}", path.display()))?;
        std::io::copy(&mut entry, &mut file)
            .with_context(|| format!("writing {}", path.display()))?;
        paths.push(path);
    }
    Ok(paths)
}

/// Runs `img2webp -d <duration> <frame> ... -o output`, lossless (its default), looping forever.
fn run_img2webp(frames: &[std::path::PathBuf], durations: &[u32], output: &Path) -> Result<()> {
    let mut command = Command::new("img2webp");
    command.args(["-loop", INFINITE_LOOP, "-m", COMPRESSION_METHOD]);
    for (frame, duration) in frames.iter().zip(durations) {
        command.args(["-d", &duration.to_string()]).arg(frame);
    }
    command.args(["-o"]).arg(output);
    let status = command
        .status()
        .context("running img2webp — install libwebp's command-line tools")?;
    ensure!(status.success(), "img2webp exited with {status}");
    Ok(())
}
