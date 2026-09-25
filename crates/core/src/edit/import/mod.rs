//! PNG images and sprite sheets, decoded under the import limits and reduced to the palette.

mod decode;
mod indexed_image;
mod quantize;
mod sheet;

pub(crate) use indexed_image::IndexedImage;
pub(crate) use sheet::plan as plan_sheet;

use super::EditError;
use crate::model::Palette;

/// The PNG file `png`, reduced to `palette`.
pub(crate) fn indexed(png: &[u8], palette: &Palette) -> Result<IndexedImage, EditError> {
    let image = decode::decode(png)?;
    Ok(IndexedImage::new(&image, palette))
}
