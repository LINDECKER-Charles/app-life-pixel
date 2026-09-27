use crate::{DecodeError, varint};

/// A cursor over the bytes still to read. Every read is bounds-checked: running out of bytes is
/// [`DecodeError::Malformed`], never a panic.
#[derive(Clone, Copy, Debug)]
pub(super) struct Reader<'a> {
    rest: &'a [u8],
}

impl<'a> Reader<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    /// The bytes not read yet.
    pub(super) fn rest(&self) -> &'a [u8] {
        self.rest
    }

    /// The bytes read since `start`, a copy of this reader taken earlier.
    pub(super) fn read_since(&self, start: Reader<'a>) -> &'a [u8] {
        let len = start.rest.len().saturating_sub(self.rest.len());
        start.rest.get(..len).unwrap_or_default()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.rest.is_empty()
    }

    /// The next `len` bytes.
    pub(super) fn bytes(&mut self, len: usize) -> Result<&'a [u8], DecodeError> {
        let (head, rest) = self
            .rest
            .split_at_checked(len)
            .ok_or(DecodeError::Malformed)?;
        self.rest = rest;
        Ok(head)
    }

    pub(super) fn u8(&mut self) -> Result<u8, DecodeError> {
        self.array().map(u8::from_le_bytes)
    }

    pub(super) fn u16(&mut self) -> Result<u16, DecodeError> {
        self.array().map(u16::from_le_bytes)
    }

    pub(super) fn u32(&mut self) -> Result<u32, DecodeError> {
        self.array().map(u32::from_le_bytes)
    }

    /// The next `len` bytes, as UTF-8.
    pub(super) fn str(&mut self, len: usize) -> Result<&'a str, DecodeError> {
        let bytes = self.bytes(len)?;
        core::str::from_utf8(bytes).map_err(|_| DecodeError::Malformed)
    }

    /// The next LEB128 varint.
    pub(super) fn varint(&mut self) -> Result<u32, DecodeError> {
        let (value, rest) = varint::read(self.rest)?;
        self.rest = rest;
        Ok(value)
    }

    pub(super) fn array<const N: usize>(&mut self) -> Result<[u8; N], DecodeError> {
        let bytes = self.bytes(N)?;
        bytes.try_into().map_err(|_| DecodeError::Malformed)
    }
}
