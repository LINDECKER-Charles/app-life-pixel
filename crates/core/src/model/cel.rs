mod cel_shape;
mod point;

pub use cel_shape::CelShape;
pub use point::Point;

/// The pixels of one layer on one frame: `width × height` palette indices, row by row from the
/// top left. Its animation checks its size and indices against a [`CelShape`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Cel(Box<[u8]>);

impl Cel {
    /// The cel of `indices`, row by row.
    #[must_use]
    pub fn new(indices: Vec<u8>) -> Self {
        Self(indices.into_boxed_slice())
    }

    /// The palette indices, row by row.
    #[must_use]
    pub fn indices(&self) -> &[u8] {
        &self.0
    }

    /// Whether every pixel is index 0, transparent: an animation stores no such cel.
    #[must_use]
    pub fn is_blank(&self) -> bool {
        self.0.iter().all(|&index| index == 0)
    }
}
