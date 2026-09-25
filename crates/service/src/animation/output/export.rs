//! `export`: every format through `compiler`, and `export_completed` with the source `mcp`.

use life_pixel_compiler::{
    ClassicOptions, ExportFile, ExportFormat, LOADER_JS, export_apng, export_gif,
    export_png_frames, export_sprite_sheet, export_wasm, file_stem,
};
use life_pixel_core::Animation;

use super::{LOADER_FILE_NAME, checked_tag};
use crate::animation::change::blocking;
use crate::animation::{AnimationEditing, EditingError};
use crate::ids::AnimationId;
use crate::owner::Owner;
use crate::ports::ProductEvent;

/// An export finished; properties [`FORMAT`], [`SIZE`] and [`SOURCE`].
const EXPORT_COMPLETED: &str = "export_completed";
/// The format exported.
const FORMAT: &str = "format";
/// The size class of the files exported, together.
const SIZE: &str = "size";
/// Where the export was asked from.
const SOURCE: &str = "source";
/// The source of the agents' exports.
const MCP: &str = "mcp";
/// The media type of a WebAssembly module.
const WASM_MEDIA_TYPE: &str = "application/wasm";
/// The media type of the loader.
const JAVASCRIPT_MEDIA_TYPE: &str = "text/javascript";

/// What `export` exports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportRequest {
    /// The animation.
    pub id: AnimationId,
    /// The format.
    pub format: ExportFormat,
    /// A tag whose frames to export; for WebAssembly, a tag the animation must have.
    pub tag: Option<String>,
    /// The scale of the classic formats; the export's own size when `None`. WebAssembly plays
    /// at any size and takes none.
    pub scale: Option<u8>,
}

impl AnimationEditing {
    /// The files of an export: WebAssembly gives `<stem>.wasm` and `life-pixel.js`; GIF, APNG and
    /// PNG frames one file; a sprite sheet its PNG and its JSON. Records `export_completed`.
    ///
    /// # Errors
    ///
    /// `export.scale`, `export.too_large` or `export.tag_not_found` from `compiler`; the
    /// library's codes.
    pub async fn export(
        &self,
        owner: &Owner,
        request: ExportRequest,
    ) -> Result<Vec<ExportFile>, EditingError> {
        let (_, animation) = self.read(owner, request.id).await?;
        let format = request.format;
        let files = blocking(move || export_files(&animation, request)).await?;
        self.record_export(owner, (format, &files));
        Ok(files)
    }

    /// Records `export_completed` for an account; the local library sends nothing.
    fn record_export(&self, owner: &Owner, (format, files): (ExportFormat, &[ExportFile])) {
        let Some(account) = owner.account() else {
            return;
        };
        let bytes = files.iter().map(|file| file.bytes.len() as u64).sum();
        let properties = vec![
            (FORMAT, format_name(format).to_owned()),
            (SIZE, ProductEvent::size_class(bytes).to_owned()),
            (SOURCE, MCP.to_owned()),
        ];
        self.events.record(ProductEvent {
            name: EXPORT_COMPLETED,
            account: Some(account),
            properties,
        });
    }
}

fn export_files(
    animation: &Animation,
    request: ExportRequest,
) -> Result<Vec<ExportFile>, EditingError> {
    let options = ClassicOptions {
        tag: request.tag,
        scale: request.scale.unwrap_or(ClassicOptions::default().scale),
    };
    let files = match request.format {
        ExportFormat::Wasm => wasm_files(animation, options.tag)?,
        ExportFormat::Gif => vec![export_gif(animation, &options)?],
        ExportFormat::Apng => vec![export_apng(animation, &options)?],
        ExportFormat::SpriteSheet => export_sprite_sheet(animation, &options)?,
        ExportFormat::PngFrames => vec![export_png_frames(animation, &options)?],
    };
    Ok(files)
}

/// The module and its loader, once `tag` is found.
fn wasm_files(animation: &Animation, tag: Option<String>) -> Result<Vec<ExportFile>, EditingError> {
    checked_tag(animation, tag)?;
    let module = ExportFile {
        name: format!("{}.wasm", file_stem(animation.title().as_str())),
        media_type: WASM_MEDIA_TYPE,
        bytes: export_wasm(animation)?,
    };
    let loader = ExportFile {
        name: LOADER_FILE_NAME.to_owned(),
        media_type: JAVASCRIPT_MEDIA_TYPE,
        bytes: LOADER_JS.as_bytes().to_vec(),
    };
    Ok(vec![module, loader])
}

/// The `format` property: the name the API and the MCP tools use.
fn format_name(format: ExportFormat) -> &'static str {
    match format {
        ExportFormat::Wasm => "wasm",
        ExportFormat::Gif => "gif",
        ExportFormat::Apng => "apng",
        ExportFormat::SpriteSheet => "sprite_sheet",
        ExportFormat::PngFrames => "png_frames",
    }
}
