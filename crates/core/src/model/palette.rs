mod rgba;

pub use rgba::Rgba;

use crate::error::DocumentError;
use crate::limits::MAX_PALETTE_ENTRIES;

/// The palette of a new animation when none is given: transparent, then 15 opaque colours.
pub const DEFAULT_PALETTE: [Rgba; 16] = [
    Rgba::from_u32(0x0000_0000),
    Rgba::from_u32(0x0000_00ff),
    Rgba::from_u32(0xffff_ffff),
    Rgba::from_u32(0x7f7f_7fff),
    Rgba::from_u32(0xc3c3_c3ff),
    Rgba::from_u32(0x8800_15ff),
    Rgba::from_u32(0xed1c_24ff),
    Rgba::from_u32(0xff7f_27ff),
    Rgba::from_u32(0xfff2_00ff),
    Rgba::from_u32(0x22b1_4cff),
    Rgba::from_u32(0x00a2_e8ff),
    Rgba::from_u32(0x3f48_ccff),
    Rgba::from_u32(0xa349_a4ff),
    Rgba::from_u32(0xb97a_57ff),
    Rgba::from_u32(0xffae_c9ff),
    Rgba::from_u32(0x99d9_eaff),
];

/// The colours of an animation, indexed from 0: 1 to [`MAX_PALETTE_ENTRIES`] entries, entry 0
/// transparent, always.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Palette(Vec<Rgba>);

impl Palette {
    /// The palette of `entries`, in order.
    ///
    /// # Errors
    ///
    /// [`DocumentError::Palette`] when there is no entry, more than [`MAX_PALETTE_ENTRIES`], or
    /// entry 0 is not transparent.
    pub fn new(entries: Vec<Rgba>) -> Result<Self, DocumentError> {
        let is_first_transparent = entries.first().is_some_and(|first| first.is_transparent());
        let is_valid = is_first_transparent && entries.len() <= MAX_PALETTE_ENTRIES;
        is_valid
            .then_some(Self(entries))
            .ok_or(DocumentError::Palette)
    }

    /// The entries, from index 0.
    #[must_use]
    pub fn entries(&self) -> &[Rgba] {
        &self.0
    }

    /// The number of entries, 1 to [`MAX_PALETTE_ENTRIES`].
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Always false: a palette holds entry 0 at least.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The colour of `index`, when the palette has it.
    #[must_use]
    pub fn get(&self, index: u8) -> Option<Rgba> {
        self.0.get(usize::from(index)).copied()
    }
}

impl Default for Palette {
    /// [`DEFAULT_PALETTE`].
    fn default() -> Self {
        Self(DEFAULT_PALETTE.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_palette_holds_1_to_256_entries_from_a_transparent_one() {
        let transparent = Rgba::default();
        let opaque = Rgba::from_u32(0x0000_00ff);
        assert!(Palette::new(vec![transparent]).is_ok());
        assert!(Palette::new(vec![transparent; MAX_PALETTE_ENTRIES]).is_ok());
        for entries in [
            vec![],
            vec![transparent; MAX_PALETTE_ENTRIES + 1],
            vec![opaque],
        ] {
            assert_eq!(Palette::new(entries), Err(DocumentError::Palette));
        }
    }

    #[test]
    fn the_default_palette_is_16_entries_from_transparent() {
        let palette = Palette::default();
        assert_eq!(palette.len(), 16);
        assert_eq!(palette.get(0), Some(Rgba::default()));
        assert_eq!(
            palette.get(15).map(|entry| entry.to_string()).unwrap(),
            "#99d9eaff"
        );
        assert_eq!(palette.get(16), None);
    }
}
