//! The model: an animation and its parts. Every type here is valid by construction; fields are
//! private and read through getters.

mod animation;
mod cel;
mod frame;
mod layer;
mod names;
mod palette;
mod project;
mod tag;

pub(crate) use animation::AnimationParts;
pub use animation::{Animation, NewAnimation};
pub use cel::{Cel, CelShape, Point};
pub use frame::{Frame, FrameId};
pub use layer::{Layer, LayerId};
pub use names::{Name, TagName};
pub use palette::{DEFAULT_PALETTE, Palette, Rgba};
pub use project::Project;
pub use tag::{LoopMode, Tag};
