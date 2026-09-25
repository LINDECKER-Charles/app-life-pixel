//! The encoder, behind the `encode` feature: an animation's flattened frames to payload v1
//! bytes, deterministically.

mod animation_data;
mod encode_error;
mod encode_payload;
mod frame_data;
mod tag_data;

pub use animation_data::AnimationData;
pub use encode_error::EncodeError;
pub use encode_payload::encode;
pub use frame_data::FrameData;
pub use tag_data::TagData;
