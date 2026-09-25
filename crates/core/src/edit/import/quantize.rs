//! An RGBA colour reduced to the palette: below [`OPAQUE_ALPHA`], index 0; otherwise the nearest
//! entry of alpha at least [`OPAQUE_ALPHA`] by squared distance on red, green and blue, the lowest
//! index winning ties.

use std::collections::BTreeMap;

use crate::model::{Palette, Rgba};

/// The alpha from which a pixel, or a palette entry, counts as opaque.
const OPAQUE_ALPHA: u8 = 128;
/// The index of a transparent pixel.
const TRANSPARENT_INDEX: u8 = 0;

/// Reduces colours to one palette, remembering each colour it has reduced.
pub(super) struct Quantizer {
    /// The opaque entries and their indices, lowest index first.
    candidates: Vec<(u8, Rgba)>,
    known: BTreeMap<[u8; 3], u8>,
}

impl Quantizer {
    /// A quantizer to `palette`.
    pub(super) fn new(palette: &Palette) -> Self {
        let entries = palette.entries().iter().copied();
        let candidates = (0..=u8::MAX)
            .zip(entries)
            .filter(|(_, colour)| colour.a >= OPAQUE_ALPHA)
            .collect();
        Self {
            candidates,
            known: BTreeMap::new(),
        }
    }

    /// The palette index of the pixel `[red, green, blue, alpha]`.
    pub(super) fn index_of(&mut self, [red, green, blue, alpha]: [u8; 4]) -> u8 {
        if alpha < OPAQUE_ALPHA {
            return TRANSPARENT_INDEX;
        }
        let key = [red, green, blue];
        if let Some(&index) = self.known.get(&key) {
            return index;
        }
        let index = self.nearest(key);
        self.known.insert(key, index);
        index
    }

    fn nearest(&self, colour: [u8; 3]) -> u8 {
        let mut best = (u32::MAX, TRANSPARENT_INDEX);
        for &(index, entry) in &self.candidates {
            let distance = squared_distance(colour, [entry.r, entry.g, entry.b]);
            if distance < best.0 {
                best = (distance, index);
            }
        }
        best.1
    }
}

fn squared_distance(left: [u8; 3], right: [u8; 3]) -> u32 {
    left.iter()
        .zip(right)
        .map(|(&left, right)| u32::from(left.abs_diff(right)).pow(2))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quantizer(colours: &[u32]) -> Quantizer {
        let entries = colours
            .iter()
            .map(|&colour| Rgba::from_u32(colour))
            .collect();
        Quantizer::new(&Palette::new(entries).unwrap())
    }

    #[test]
    fn a_pixel_below_half_alpha_is_transparent() {
        let mut quantizer = quantizer(&[0, 0xff00_00ff]);
        assert_eq!(quantizer.index_of([255, 0, 0, 127]), 0);
        assert_eq!(quantizer.index_of([255, 0, 0, 128]), 1);
    }

    #[test]
    fn the_nearest_opaque_entry_wins_the_lowest_index_on_ties() {
        let mut quantizer = quantizer(&[0, 0x0000_00ff, 0xffff_ff7f, 0x6464_64ff, 0x9c9c_9cff]);
        assert_eq!(
            quantizer.index_of([250, 250, 250, 255]),
            4,
            "the translucent white is skipped"
        );
        assert_eq!(
            quantizer.index_of([128, 128, 128, 255]),
            3,
            "a tie goes to the lowest index"
        );
        assert_eq!(quantizer.index_of([20, 20, 20, 200]), 1);
    }

    #[test]
    fn without_an_opaque_entry_everything_is_transparent() {
        let mut quantizer = quantizer(&[0]);
        assert_eq!(quantizer.index_of([255, 255, 255, 255]), 0);
    }
}
