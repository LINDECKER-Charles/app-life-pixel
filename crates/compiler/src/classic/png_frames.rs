//! PNG frames: a zip of one indexed PNG per frame, in play order.

use std::io::{Cursor, Write};

use life_pixel_core::Animation;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

use super::ClassicOptions;
use super::plan::Plan;
use crate::naming::frame_file_name;
use crate::{ExportError, ExportFile};

/// The media type of a zip.
const ZIP_MEDIA_TYPE: &str = "application/zip";

/// Every frame of the range as `<stem>-<position>.png`, in `<stem>-frames.zip`. The PNGs are
/// stored, since they are compressed already, and dated 1980-01-01 — the zip's earliest date —,
/// so the same animation always gives the same zip.
///
/// # Errors
///
/// [`ExportError::Scale`], [`ExportError::TagNotFound`] or [`ExportError::TooLarge`] when
/// `options` do not fit the animation.
pub fn export_png_frames(
    animation: &Animation,
    options: &ClassicOptions,
) -> Result<ExportFile, ExportError> {
    let plan = Plan::new(animation, options)?;
    let stem = plan.stem();
    let entry_options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(DateTime::default());
    let frame_count = plan.range.frames.len();
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    for (position, frame) in plan.range.frames.iter().enumerate() {
        let indices = plan.render(frame);
        let png = plan.frame_image().encode(&indices)?;
        let name = frame_file_name(&stem, position, frame_count);
        zip.start_file(name, entry_options)
            .map_err(ExportError::encoding)?;
        zip.write_all(&png).map_err(ExportError::encoding)?;
    }
    let bytes = zip.finish().map_err(ExportError::encoding)?.into_inner();
    Ok(ExportFile {
        name: format!("{stem}-frames.zip"),
        media_type: ZIP_MEDIA_TYPE,
        bytes,
    })
}
