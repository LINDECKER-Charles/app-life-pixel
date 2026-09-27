use super::decode::{RGBA_BYTES, RgbaImage};
use super::quantize::Quantizer;
use crate::edit::Area;
use crate::edit::pixels::Canvas;
use crate::model::{Palette, Point};

/// An imported image reduced to a palette: one index per pixel, row by row.
pub(crate) struct IndexedImage {
    width: u32,
    height: u32,
    indices: Vec<u8>,
}

/// Which part of an image goes where on a canvas.
pub(crate) struct Placement {
    /// The part of the image pasted.
    pub(crate) source: Area,
    /// Where its top-left pixel goes.
    pub(crate) at: Point,
}

impl IndexedImage {
    /// `image` reduced to `palette`.
    pub(super) fn new(image: &RgbaImage, palette: &Palette) -> Self {
        let mut quantizer = Quantizer::new(palette);
        let indices = image
            .pixels
            .as_chunks::<RGBA_BYTES>()
            .0
            .iter()
            .map(|&pixel| quantizer.index_of(pixel))
            .collect();
        Self {
            width: image.width,
            height: image.height,
            indices,
        }
    }

    /// The width, in pixels.
    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    /// The height, in pixels.
    pub(crate) fn height(&self) -> u32 {
        self.height
    }

    /// The whole image.
    pub(crate) fn area(&self) -> Area {
        Area {
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
        }
    }

    /// Whether every pixel of `area`, inside the image, is index 0.
    pub(crate) fn is_blank_in(&self, area: Area) -> bool {
        self.pixels_in(area).all(|(_, _, index)| index == 0)
    }

    /// Pastes the whole image's non-zero pixels on the canvas, its top-left pixel at `at`,
    /// clipped.
    pub(crate) fn paste_whole(&self, canvas: &mut Canvas, at: Point) {
        let placement = Placement {
            source: self.area(),
            at,
        };
        self.paste(canvas, &placement);
    }

    /// Pastes the non-zero pixels of the placement's source on the canvas, clipped.
    pub(crate) fn paste(&self, canvas: &mut Canvas, placement: &Placement) {
        let shift_x = i64::from(placement.at.x) - i64::from(placement.source.x);
        let shift_y = i64::from(placement.at.y) - i64::from(placement.source.y);
        for (x, y, index) in self.pixels_in(placement.source) {
            if index != 0 {
                canvas.set((x + shift_x, y + shift_y), index);
            }
        }
    }

    /// The `(x, y, index)` of each pixel of `area` inside the image.
    fn pixels_in(&self, area: Area) -> impl Iterator<Item = (i64, i64, u8)> + '_ {
        let (width, height) = (i64::from(self.width), i64::from(self.height));
        let (left, top) = (i64::from(area.x), i64::from(area.y));
        let columns = left.max(0)..(left + i64::from(area.width)).min(width);
        let rows = top.max(0)..(top + i64::from(area.height)).min(height);
        rows.flat_map(move |y| columns.clone().map(move |x| (x, y)))
            .filter_map(move |(x, y)| {
                let offset = usize::try_from(y * width + x).ok()?;
                Some((x, y, *self.indices.get(offset)?))
            })
    }
}
