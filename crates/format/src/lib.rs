//! The versioned binary format of a compiled Life Pixel animation: payload v1 and player ABI v1.
//!
//! `README.md` in this crate specifies both, byte by byte. The decoder is `no_std` and never
//! allocates; the encoder, behind the `encode` feature, uses `alloc`.

#![no_std]

#[cfg(feature = "encode")]
extern crate alloc;

pub mod bounds;
mod decode;
#[cfg(feature = "encode")]
mod encode;
mod frame;
mod frame_kind;
mod loop_mode;
mod rgba;
mod tag;

pub use decode::{DecodeError, Payload, apply_frame};
#[cfg(feature = "encode")]
pub use encode::{AnimationData, EncodeError, FrameData, TagData, encode};
pub use frame::Frame;
pub use frame_kind::FrameKind;
pub use loop_mode::LoopMode;
pub use rgba::Rgba;
pub use tag::Tag;

/// The first four bytes of every payload: `LPIX`.
pub const MAGIC: [u8; 4] = *b"LPIX";

/// The payload format version this crate reads and writes.
pub const FORMAT_VERSION: u16 = 1;

/// The player ABI version a payload of this crate is written for.
pub const ABI_VERSION: u16 = 1;
