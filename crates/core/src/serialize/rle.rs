//! A cel's `rle`: base64, with padding, of pairs *(LEB128 run length ≥ 1, index byte)* covering
//! the cel exactly.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::error::DocumentError;

/// The low seven bits of a LEB128 byte carry the value.
const LEB128_VALUE_BITS: u32 = 7;
/// The high bit of a LEB128 byte says another byte follows.
const LEB128_CONTINUE: u8 = 0x80;
/// The most bytes of a run length: 35 bits, far above any cel.
const LEB128_MAX_BYTES: u32 = 5;

/// The `rle` text of `indices`, in maximal runs.
pub(crate) fn encode(indices: &[u8]) -> String {
    let mut bytes = Vec::new();
    for run in indices.chunk_by(|left, right| left == right) {
        write_run_length(run.len(), &mut bytes);
        bytes.push(run[0]);
    }
    STANDARD.encode(bytes)
}

/// The `pixel_count` indices of the `rle` text.
///
/// Fails with [`DocumentError::Cel`] when the text is not padded base64, a run is empty or
/// truncated, or the runs do not cover `pixel_count` pixels exactly.
pub(crate) fn decode(text: &str, pixel_count: usize) -> Result<Vec<u8>, DocumentError> {
    let bytes = STANDARD.decode(text).map_err(|_| DocumentError::Cel)?;
    let mut bytes = bytes.into_iter();
    let mut indices = Vec::with_capacity(pixel_count);
    while !bytes.as_slice().is_empty() {
        let length = read_run_length(&mut bytes).ok_or(DocumentError::Cel)?;
        let index = bytes.next().ok_or(DocumentError::Cel)?;
        let remaining = pixel_count - indices.len();
        if length == 0 || length > remaining {
            return Err(DocumentError::Cel);
        }
        indices.resize(indices.len() + length, index);
    }
    (indices.len() == pixel_count)
        .then_some(indices)
        .ok_or(DocumentError::Cel)
}

fn write_run_length(length: usize, bytes: &mut Vec<u8>) {
    let mut rest = length;
    loop {
        let low_bits = rest.to_le_bytes()[0] & !LEB128_CONTINUE;
        rest >>= LEB128_VALUE_BITS;
        if rest == 0 {
            bytes.push(low_bits);
            return;
        }
        bytes.push(low_bits | LEB128_CONTINUE);
    }
}

fn read_run_length(bytes: &mut impl Iterator<Item = u8>) -> Option<usize> {
    let mut value: u64 = 0;
    for position in 0..LEB128_MAX_BYTES {
        let byte = bytes.next()?;
        value |= u64::from(byte & !LEB128_CONTINUE) << (position * LEB128_VALUE_BITS);
        if byte & LEB128_CONTINUE == 0 {
            return usize::try_from(value).ok();
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_are_leb128_lengths_then_indices() {
        let mut indices = vec![1; 200];
        indices.push(0);
        let bytes = STANDARD.decode(encode(&indices)).unwrap();
        assert_eq!(bytes, [0xc8, 0x01, 1, 0x01, 0]);
        assert_eq!(decode(&encode(&indices), 201).unwrap(), indices);
    }

    #[test]
    fn runs_must_cover_the_cel_exactly() {
        let two_ones = STANDARD.encode([2, 1]);
        assert_eq!(decode(&two_ones, 2).unwrap(), [1, 1]);
        for pixel_count in [1, 3] {
            assert_eq!(decode(&two_ones, pixel_count), Err(DocumentError::Cel));
        }
    }

    #[test]
    fn an_empty_truncated_or_unpadded_run_is_refused() {
        let empty_run = STANDARD.encode([0, 1, 2, 1]);
        let truncated = STANDARD.encode([2]);
        let endless = STANDARD.encode([0xff; 6]);
        for text in [&empty_run, &truncated, &endless, "AgE", "not base64!"] {
            assert_eq!(decode(text, 2), Err(DocumentError::Cel), "{text:?}");
        }
    }
}
