use super::Cel;

/// What a cel of an animation must fit: its canvas size and its palette size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CelShape {
    /// The canvas width, in pixels.
    pub width: u16,
    /// The canvas height, in pixels.
    pub height: u16,
    /// The number of palette entries: every index is below it.
    pub palette_len: usize,
}

impl CelShape {
    /// The number of pixels of a cel: `width × height`.
    #[must_use]
    pub fn pixel_count(&self) -> usize {
        usize::from(self.width) * usize::from(self.height)
    }

    /// Whether `cel` has [`pixel_count`](Self::pixel_count) pixels, each in the palette.
    #[must_use]
    pub fn fits(&self, cel: &Cel) -> bool {
        let indices = cel.indices();
        indices.len() == self.pixel_count()
            && indices
                .iter()
                .all(|&index| usize::from(index) < self.palette_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_cel_fits_by_size_and_palette() {
        let shape = CelShape {
            width: 2,
            height: 1,
            palette_len: 3,
        };
        assert!(shape.fits(&Cel::new(vec![0, 2])));
        assert!(!shape.fits(&Cel::new(vec![0, 3])));
        assert!(!shape.fits(&Cel::new(vec![0, 1, 2])));
    }
}
