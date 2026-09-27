use crate::model::{Animation, Cel, FrameId, LayerId, Point};

/// A pixel's column and row, from the top left of the canvas.
pub(crate) type Pixel = (i64, i64);

/// A cel being drawn on: its palette indices, row by row, and the canvas size. Coordinates are
/// wide, so that no offset or clipping overflows; pixels outside the canvas are ignored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Canvas {
    width: i64,
    height: i64,
    indices: Vec<u8>,
}

impl Canvas {
    /// A copy of the cel of `layer` on `frame`, blank when it has none.
    pub(crate) fn of(animation: &Animation, layer: LayerId, frame: FrameId) -> Self {
        let shape = animation.cel_shape();
        let indices = animation.cel(layer, frame).map_or_else(
            || vec![0; shape.pixel_count()],
            |cel| cel.indices().to_vec(),
        );
        Self {
            width: i64::from(shape.width),
            height: i64::from(shape.height),
            indices,
        }
    }

    /// A blank cel of `animation`'s canvas size.
    pub(crate) fn blank(animation: &Animation) -> Self {
        let shape = animation.cel_shape();
        Self {
            width: i64::from(shape.width),
            height: i64::from(shape.height),
            indices: vec![0; shape.pixel_count()],
        }
    }

    /// The canvas width, in pixels.
    pub(crate) fn width(&self) -> i64 {
        self.width
    }

    /// The canvas height, in pixels.
    pub(crate) fn height(&self) -> i64 {
        self.height
    }

    /// Whether `point` lies on the canvas.
    pub(crate) fn contains(&self, point: Point) -> bool {
        self.offset(i64::from(point.x), i64::from(point.y))
            .is_some()
    }

    /// The index of `pixel`, when it lies on the canvas.
    pub(crate) fn get(&self, (x, y): Pixel) -> Option<u8> {
        self.offset(x, y).map(|offset| self.indices[offset])
    }

    /// Paints `pixel` with `index`; nothing happens outside the canvas.
    pub(crate) fn set(&mut self, (x, y): Pixel, index: u8) {
        if let Some(offset) = self.offset(x, y) {
            self.indices[offset] = index;
        }
    }

    /// The finished cel.
    pub(crate) fn into_cel(self) -> Cel {
        Cel::new(self.indices)
    }

    fn offset(&self, x: i64, y: i64) -> Option<usize> {
        let is_inside = (0..self.width).contains(&x) && (0..self.height).contains(&y);
        is_inside
            .then(|| usize::try_from(y * self.width + x).ok())
            .flatten()
    }
}
