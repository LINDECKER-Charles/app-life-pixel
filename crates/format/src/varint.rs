//! The unsigned LEB128 varint of extended operation counts: 7 bits per byte, least significant
//! group first, bit 7 set on every byte but the last, at most [`MAX_BYTES`] bytes.

use crate::DecodeError;

/// The longest varint payload v1 accepts, in bytes.
pub(crate) const MAX_BYTES: usize = 5;

/// Bit 7: more bytes follow.
const CONTINUATION: u8 = 0x80;

/// Bits 6–0: the group of bits a byte carries.
const GROUP_MASK: u8 = 0x7F;

/// The number of bits a byte carries.
const GROUP_BITS: u32 = 7;

/// Reads the varint at the start of `bytes`: its value and the bytes after it.
///
/// # Errors
///
/// [`DecodeError::Malformed`] when `bytes` ends inside the varint, when the varint runs past
/// [`MAX_BYTES`], or when its value does not fit a `u32` — a count that large passes any canvas.
pub(crate) fn read(bytes: &[u8]) -> Result<(u32, &[u8]), DecodeError> {
    let mut value: u64 = 0;
    for (position, &byte) in bytes.iter().take(MAX_BYTES).enumerate() {
        let shift = GROUP_BITS * u32::try_from(position).map_err(|_| DecodeError::Malformed)?;
        value |= u64::from(byte & GROUP_MASK) << shift;
        if byte & CONTINUATION == 0 {
            let value = u32::try_from(value).map_err(|_| DecodeError::Malformed)?;
            let rest = bytes.get(position + 1..).ok_or(DecodeError::Malformed)?;
            return Ok((value, rest));
        }
    }
    Err(DecodeError::Malformed)
}

/// Appends `value` to `out` as a varint, in as few bytes as it takes.
#[cfg(feature = "encode")]
pub(crate) fn write(value: usize, out: &mut alloc::vec::Vec<u8>) {
    let mut rest = value;
    loop {
        let group = (rest & usize::from(GROUP_MASK)) as u8;
        rest >>= GROUP_BITS;
        if rest == 0 {
            out.push(group);
            return;
        }
        out.push(group | CONTINUATION);
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_BYTES, read};
    use crate::DecodeError;

    #[test]
    fn reads_one_byte_and_returns_the_rest() {
        assert_eq!(read(&[0x05, 0xAA]), Ok((5, &[0xAA][..])));
    }

    #[test]
    fn reads_the_readme_example() {
        assert_eq!(read(&[0xEC, 0x01]), Ok((236, &[][..])));
    }

    #[test]
    fn reads_five_bytes_up_to_u32_max() {
        assert_eq!(
            read(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]),
            Ok((u32::MAX, &[][..]))
        );
    }

    #[test]
    fn accepts_a_non_minimal_five_byte_varint() {
        assert_eq!(read(&[0x80, 0x80, 0x80, 0x80, 0x00]), Ok((0, &[][..])));
    }

    #[test]
    fn refuses_a_varint_of_six_bytes() {
        let bytes = [0x80, 0x80, 0x80, 0x80, 0x80, 0x00];
        assert_eq!(bytes.len(), MAX_BYTES + 1);
        assert_eq!(read(&bytes), Err(DecodeError::Malformed));
    }

    #[test]
    fn refuses_a_value_beyond_u32() {
        assert_eq!(
            read(&[0xFF, 0xFF, 0xFF, 0xFF, 0x10]),
            Err(DecodeError::Malformed)
        );
    }

    #[test]
    fn refuses_a_truncated_varint() {
        assert_eq!(read(&[0x80, 0x80]), Err(DecodeError::Malformed));
        assert_eq!(read(&[]), Err(DecodeError::Malformed));
    }

    #[cfg(feature = "encode")]
    #[test]
    fn written_values_read_back() {
        for value in [
            0,
            1,
            127,
            128,
            236,
            16_383,
            16_384,
            4_194_240,
            u32::MAX as usize,
        ] {
            let mut bytes = alloc::vec::Vec::new();
            super::write(value, &mut bytes);
            assert_eq!(read(&bytes), Ok((value as u32, &[][..])));
        }
    }

    #[cfg(feature = "encode")]
    #[test]
    fn writes_the_readme_example() {
        let mut bytes = alloc::vec::Vec::new();
        super::write(236, &mut bytes);
        assert_eq!(bytes, [0xEC, 0x01]);
    }
}
