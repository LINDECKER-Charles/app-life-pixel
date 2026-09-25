//! The fixed values of payload v1 that both the decoder and the encoder read: the reserved
//! field, the palette's limits and the operations of frame data.

/// The value of the header's reserved field.
pub(crate) const RESERVED: u16 = 0;

/// The largest palette: every index a byte can hold.
pub(crate) const MAX_PALETTE_ENTRIES: u16 = 256;

/// The size of a palette entry: red, green, blue, alpha.
pub(crate) const PALETTE_ENTRY_BYTES: usize = 4;

/// Palette entry 0: fully transparent.
pub(crate) const TRANSPARENT: [u8; PALETTE_ENTRY_BYTES] = [0; PALETTE_ENTRY_BYTES];

/// Bits 7–6 of an operation byte: its operation.
pub(crate) const OPERATION_SHIFT: u32 = 6;

/// Bits 5–0 of an operation byte: `n`.
pub(crate) const COUNT_MASK: u8 = 0x3F;

/// The `n` that announces a varint: the count is [`EXTENDED_COUNT_BASE`] plus its value.
pub(crate) const EXTENDED_COUNT_MARKER: u8 = 63;

/// The smallest count a varint carries, and the first that does not fit in `n`.
pub(crate) const EXTENDED_COUNT_BASE: usize = 64;

/// The operation codes of bits 7–6.
pub(crate) mod operation {
    /// The next pixels keep the previous frame's indices; delta frames only.
    pub(crate) const SKIP: u8 = 0b00;
    /// The next pixels take one index.
    pub(crate) const RUN: u8 = 0b01;
    /// The next pixels take the indices that follow, in order.
    pub(crate) const LITERAL: u8 = 0b10;
}
