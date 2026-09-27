//! A WebAssembly custom section, appended after a module's last section: custom sections may
//! appear anywhere, so the module stays valid and its code unchanged.

use crate::ExportError;

/// The id of a custom section.
const CUSTOM_SECTION_ID: u8 = 0x00;
/// The bits of a value each LEB128 byte carries.
const LEB128_BITS: u32 = 7;
/// The bits of a LEB128 byte that carry the value.
const LEB128_VALUE_MASK: u32 = 0x7f;
/// The bit of a LEB128 byte set when another byte follows.
const LEB128_CONTINUATION: u8 = 0x80;

/// Appends to `module` the custom section `name` holding `content`: byte `0x00`, the LEB128 size
/// of what follows, the LEB128 length of `name`, `name`, then `content`.
///
/// # Errors
///
/// [`ExportError::Encoding`] when the section passes the 4 GiB a WebAssembly size can state.
pub(super) fn append(module: &mut Vec<u8>, name: &str, content: &[u8]) -> Result<(), ExportError> {
    let name_length = u32_length(name.len())?;
    let mut body = leb128(name_length);
    body.extend_from_slice(name.as_bytes());
    let size = u32_length(body.len() + content.len())?;
    module.push(CUSTOM_SECTION_ID);
    module.extend(leb128(size));
    module.extend(body);
    module.extend_from_slice(content);
    Ok(())
}

/// `length` as the `u32` WebAssembly sizes are.
fn u32_length(length: usize) -> Result<u32, ExportError> {
    u32::try_from(length).map_err(ExportError::encoding)
}

/// `value` in unsigned LEB128: 7 bits per byte, the lowest first, each byte but the last with its
/// high bit set.
fn leb128(mut value: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    loop {
        let low_bits = u8::try_from(value & LEB128_VALUE_MASK).unwrap_or_default();
        value >>= LEB128_BITS;
        if value == 0 {
            bytes.push(low_bits);
            return bytes;
        }
        bytes.push(low_bits | LEB128_CONTINUATION);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leb128_writes_7_bits_per_byte_lowest_first() {
        assert_eq!(leb128(0), [0x00]);
        assert_eq!(leb128(10), [0x0a]);
        assert_eq!(leb128(127), [0x7f]);
        assert_eq!(leb128(128), [0x80, 0x01]);
        assert_eq!(leb128(624_485), [0xe5, 0x8e, 0x26]);
        assert_eq!(leb128(u32::MAX), [0xff, 0xff, 0xff, 0xff, 0x0f]);
    }

    #[test]
    fn a_section_is_its_id_its_size_its_name_and_its_content() {
        let mut module = b"head".to_vec();

        append(&mut module, "life-pixel", b"LPIX").unwrap();

        let mut expected = b"head".to_vec();
        expected.extend([0x00, 15, 10]);
        expected.extend(b"life-pixelLPIX");
        assert_eq!(module, expected);
    }

    #[test]
    fn a_section_of_128_bytes_or_more_states_its_size_in_several_bytes() {
        let content = [7; 200];
        let mut module = Vec::new();

        append(&mut module, "life-pixel", &content).unwrap();

        assert_eq!(module[..4], [0x00, 0xd3, 0x01, 10]);
        assert_eq!(module.len(), 4 + 10 + 200);
    }
}
