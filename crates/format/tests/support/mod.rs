//! Payloads written byte by byte, independently of the encoder, and the decoding of a whole
//! payload back into an animation.

#![allow(
    dead_code,
    unused_imports,
    reason = "each test crate uses a part of the support"
)]

#[cfg(feature = "encode")]
mod animations;
#[cfg(feature = "encode")]
mod decoded;

#[cfg(feature = "encode")]
pub use animations::{animation, palette, pattern, tag};
#[cfg(feature = "encode")]
pub use decoded::{decode_animation, frame_kinds};

/// The 61-byte example of `crates/format/README.md`: 2 × 2, titled `Hi`, two frames, one tag.
pub const README_EXAMPLE: [u8; 61] = [
    0x4C, 0x50, 0x49, 0x58, 0x01, 0x00, 0x01, 0x00, 0x02, 0x00, 0x02, 0x00, 0x02, 0x00, 0x00, 0x00,
    0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFF, 0x00, 0x00, 0xFF, 0x02, 0x00, 0x48, 0x69, 0x01, 0x00,
    0x00, 0x00, 0x01, 0x00, 0x00, 0x04, 0x69, 0x64, 0x6C, 0x65, 0x64, 0x00, 0x00, 0x02, 0x00, 0x00,
    0x00, 0x43, 0x01, 0x64, 0x00, 0x01, 0x03, 0x00, 0x00, 0x00, 0x02, 0x80, 0x00,
];

/// Where each part of [`README_EXAMPLE`] lies.
pub mod offset {
    use core::ops::Range;

    pub const MAGIC: usize = 0;
    pub const FORMAT_VERSION: usize = 4;
    pub const ABI_VERSION: usize = 6;
    pub const WIDTH: usize = 8;
    pub const HEIGHT: usize = 10;
    pub const FRAME_COUNT: usize = 12;
    pub const RESERVED: usize = 14;
    pub const PALETTE_COUNT: usize = 16;
    pub const PALETTE_ENTRY_0: usize = 18;
    pub const TITLE_LEN: usize = 26;
    pub const TITLE: usize = 28;
    pub const TAG_COUNT: usize = 30;
    pub const TAG_FIRST: usize = 32;
    pub const TAG_LAST: usize = 34;
    pub const TAG_LOOP_MODE: usize = 36;
    pub const TAG_NAME_LEN: usize = 37;
    pub const TAG_NAME: usize = 38;
    pub const FRAME_0_DURATION: usize = 42;
    pub const FRAME_0_KIND: usize = 44;
    pub const FRAME_0_DATA: usize = 49;
    pub const FRAME_1_KIND: usize = 53;
    pub const FRAME_1_DATA: usize = 58;

    pub const HEADER: Range<usize> = 0..16;
    pub const PALETTE: Range<usize> = 16..26;
    pub const TITLE_SECTION: Range<usize> = 26..30;
    pub const TAGS: Range<usize> = 30..42;
    pub const FRAMES: Range<usize> = 42..61;
}

/// [`README_EXAMPLE`] with the `u16` at `offset` set to `value`.
pub fn example_with_u16(offset: usize, value: u16) -> Vec<u8> {
    example_with(offset, &value.to_le_bytes())
}

/// [`README_EXAMPLE`] with `bytes` written at `offset`.
pub fn example_with(offset: usize, bytes: &[u8]) -> Vec<u8> {
    let mut payload = README_EXAMPLE.to_vec();
    payload[offset..offset + bytes.len()].copy_from_slice(bytes);
    payload
}

/// A payload over the example's palette (transparent, then red), without title or tags, whose
/// canvas and frames are given: `(kind, data)` per frame, 100 ms each.
pub fn payload_with_frames(width: u16, height: u16, frames: &[(u8, &[u8])]) -> Vec<u8> {
    let frame_count = frames.len() as u16;
    let mut payload = README_EXAMPLE[..offset::WIDTH].to_vec();
    for value in [width, height, frame_count, 0] {
        payload.extend_from_slice(&value.to_le_bytes());
    }
    payload.extend_from_slice(&README_EXAMPLE[offset::PALETTE]);
    payload.extend_from_slice(&[0, 0, 0, 0]);
    for &(kind, data) in frames {
        payload.extend_from_slice(&100_u16.to_le_bytes());
        payload.push(kind);
        payload.extend_from_slice(&(data.len() as u32).to_le_bytes());
        payload.extend_from_slice(data);
    }
    payload
}
