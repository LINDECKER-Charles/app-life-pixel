//! The Life Pixel document model: the one place where a pixel rule lives.
//!
//! An [`Animation`] is built by [`Animation::new`] or read from a document by
//! [`serialize::read_document`], so an animation in memory is always valid. [`limits`] holds every
//! product limit, [`error`] the codes a document can fail with, [`serialize`] the JSON document and
//! the text grid, [`render`] the compositing of a frame. The crate is pure: no I/O, no clock, no
//! randomness, no async.

pub mod error;
pub mod limits;
pub mod model;
pub mod render;
pub mod serialize;

pub use error::DocumentError;
pub use limits::Limits;
pub use model::{
    Animation, Cel, CelShape, DEFAULT_PALETTE, Frame, FrameId, Layer, LayerId, LoopMode, Name,
    NewAnimation, Palette, Point, Project, Rgba, Tag, TagName,
};
