//! The decoder: parsing and checking a payload, then applying its frames. It never allocates
//! and never panics, whatever the input.

mod apply_frame;
mod decode_error;
mod frames;
mod header;
mod operations;
mod payload;
mod reader;
mod sections;

pub use apply_frame::apply_frame;
pub use decode_error::DecodeError;
pub use payload::Payload;
