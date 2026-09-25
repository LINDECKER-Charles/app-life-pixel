//! The decoder's bounds. They are wider than the domain limits of `core`, so that every
//! animation the editor accepts fits; a value beyond one is refused with
//! [`DecodeError::BeyondBound`](crate::DecodeError::BeyondBound).

/// The largest width or height, in pixels.
pub const MAX_SIDE: u16 = 2_048;

/// The largest number of frames.
pub const MAX_FRAMES: u16 = 4_096;

/// The largest number of tags.
pub const MAX_TAGS: u16 = 255;

/// The longest title, in UTF-8 bytes.
pub const MAX_TITLE_BYTES: u16 = 1_024;

/// The longest tag name, in UTF-8 bytes.
pub const MAX_TAG_NAME_BYTES: u8 = 64;

/// The largest payload, in bytes: 64 MiB.
pub const MAX_PAYLOAD_BYTES: u32 = 64 * 1_024 * 1_024;
