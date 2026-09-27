//! Scaling: each pixel repeated `scale × scale` times, within the export limits.

use life_pixel_core::limits::{EXPORT_MAX_SCALE, EXPORT_MAX_SIDE, EXPORT_MIN_SCALE};

use crate::ExportError;

/// `scale` itself, when it lies in [`EXPORT_MIN_SCALE`] to [`EXPORT_MAX_SCALE`].
pub(crate) fn checked(scale: u8) -> Result<u8, ExportError> {
    let bounds = EXPORT_MIN_SCALE..=EXPORT_MAX_SCALE;
    if bounds.contains(&u32::from(scale)) {
        Ok(scale)
    } else {
        Err(ExportError::Scale)
    }
}

/// `count` sides of `side` pixels end to end, when they fit in [`EXPORT_MAX_SIDE`].
pub(crate) fn bounded_side(side: u32, count: u32) -> Result<u32, ExportError> {
    side.checked_mul(count)
        .filter(|&total| total <= EXPORT_MAX_SIDE)
        .ok_or(ExportError::TooLarge)
}

/// The pixels of an image `width` pixels wide, each repeated `scale` times across and down.
pub(crate) fn upscale(pixels: &[u8], width: usize, scale: usize) -> Vec<u8> {
    if scale <= 1 {
        return pixels.to_vec();
    }
    let mut scaled = Vec::with_capacity(pixels.len() * scale * scale);
    for row in pixels.chunks(width.max(1)) {
        let wide_row: Vec<u8> = row
            .iter()
            .flat_map(|&pixel| std::iter::repeat_n(pixel, scale))
            .collect();
        for _ in 0..scale {
            scaled.extend_from_slice(&wide_row);
        }
    }
    scaled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_pixel_is_repeated_across_and_down() {
        assert_eq!(upscale(&[1, 2, 3, 4], 2, 1), [1, 2, 3, 4]);
        let expected = [1, 1, 2, 2, 1, 1, 2, 2, 3, 3, 4, 4, 3, 3, 4, 4];
        assert_eq!(upscale(&[1, 2, 3, 4], 2, 2), expected);
    }

    #[test]
    fn scales_and_sides_are_bounded() {
        assert_eq!(checked(1), Ok(1));
        assert_eq!(checked(16), Ok(16));
        assert_eq!(checked(0), Err(ExportError::Scale));
        assert_eq!(checked(17), Err(ExportError::Scale));
        assert_eq!(bounded_side(512, 16), Ok(8_192));
        assert_eq!(bounded_side(513, 16), Err(ExportError::TooLarge));
    }
}
