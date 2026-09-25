//! The files of an export, from `compiler`: the WebAssembly bundle with its loader, or the files of
//! a classic format.

use life_pixel_compiler::{
    ClassicOptions, ExportError, ExportFile, ExportFormat, LOADER_JS, export_apng, export_gif,
    export_png_frames, export_sprite_sheet, export_wasm, file_stem,
};
use life_pixel_core::Animation;
use serde::{Deserialize, Serialize};

use crate::errors::EngineError;

/// The file name of the loader: one copy serves every export of a site.
const LOADER_FILE_NAME: &str = "life-pixel.js";
/// The media type of the loader.
const LOADER_MEDIA_TYPE: &str = "text/javascript";
/// The extension of the WebAssembly bundle, after the animation's file stem.
const WASM_EXTENSION: &str = "wasm";
/// The media type of the WebAssembly bundle, which lets browsers compile it while it downloads.
const WASM_MEDIA_TYPE: &str = "application/wasm";

/// The interface's `ExportRequest`. The WebAssembly bundle holds every frame and tag at the
/// animation's size: `tag` and `scale` apply to the classic formats only.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExportRequest {
    /// The format exported to.
    pub format: ExportFormat,
    /// The tag whose frames a classic format exports; every frame when absent.
    pub tag: Option<String>,
    /// How many times a classic format repeats each pixel; 1 when absent.
    pub scale: Option<u32>,
}

/// The interface's `ExportResult`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    /// The files, in the order the format lists them.
    pub files: Vec<ExportedFile>,
    /// The bytes of every file together.
    pub total_bytes: usize,
}

/// The interface's `ExportedFile`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedFile {
    /// The file name: `mascot.gif`, for example.
    pub name: String,
    /// The media type: `image/gif`, for example.
    pub media_type: &'static str,
    /// The file's content, a `Uint8Array` in JavaScript.
    #[serde(with = "serde_bytes")]
    pub bytes: Vec<u8>,
}

impl From<ExportFile> for ExportedFile {
    fn from(file: ExportFile) -> Self {
        Self {
            name: file.name,
            media_type: file.media_type,
            bytes: file.bytes,
        }
    }
}

/// The files `request` asks for.
///
/// # Errors
///
/// The compiler's `export.*` code for a scale, a size or a tag it refuses, `internal.error` when an
/// encoder fails.
pub fn export_files(
    animation: &Animation,
    request: &ExportRequest,
) -> Result<ExportResult, EngineError> {
    let files = match request.format {
        ExportFormat::Wasm => wasm_files(animation)?,
        ExportFormat::Gif => vec![export_gif(animation, &classic_options(request)?)?],
        ExportFormat::Apng => vec![export_apng(animation, &classic_options(request)?)?],
        ExportFormat::SpriteSheet => export_sprite_sheet(animation, &classic_options(request)?)?,
        ExportFormat::PngFrames => vec![export_png_frames(animation, &classic_options(request)?)?],
    };
    let files: Vec<ExportedFile> = files.into_iter().map(ExportedFile::from).collect();
    let total_bytes = files.iter().map(|file| file.bytes.len()).sum();
    Ok(ExportResult { files, total_bytes })
}

/// The bundle, named after the animation, and the loader that plays it.
fn wasm_files(animation: &Animation) -> Result<Vec<ExportFile>, ExportError> {
    let bundle = ExportFile {
        name: format!("{}.{WASM_EXTENSION}", file_stem(animation.title().as_str())),
        media_type: WASM_MEDIA_TYPE,
        bytes: export_wasm(animation)?,
    };
    let loader = ExportFile {
        name: LOADER_FILE_NAME.to_owned(),
        media_type: LOADER_MEDIA_TYPE,
        bytes: LOADER_JS.as_bytes().to_vec(),
    };
    Ok(vec![bundle, loader])
}

/// A scale beyond what the compiler's options hold is out of its limits.
fn classic_options(request: &ExportRequest) -> Result<ClassicOptions, ExportError> {
    let defaults = ClassicOptions::default();
    let scale = request
        .scale
        .map_or(Ok(defaults.scale), u8::try_from)
        .map_err(|_| ExportError::Scale)?;
    Ok(ClassicOptions {
        tag: request.tag.clone(),
        scale,
    })
}
