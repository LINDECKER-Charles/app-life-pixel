//! The WebAssembly export: the prebuilt [player](crate::PLAYER_WASM) followed by the animation's
//! payload in a custom section — data, never code.

mod custom_section;
mod flatten;

use life_pixel_core::Animation;

use crate::{ExportError, PLAYER_WASM};

/// The custom section the loader reads the payload from.
const SECTION_NAME: &str = "life-pixel";

/// The animation as a self-contained WebAssembly module: [`PLAYER_WASM`], then a custom section
/// named `life-pixel` — byte `0x00`, the LEB128 size of what follows, the LEB128 length of the
/// name, the name, the payload. Every frame is composited by `life-pixel-core` and encoded as
/// payload v1 with the animation's palette, title and tags, so the module plays what the editor
/// shows; its code stays the player's, byte for byte. The same animation always gives the same
/// bytes.
///
/// # Errors
///
/// [`ExportError::Encoding`] when the encoder refuses the animation, which a valid animation never
/// causes: the domain limits of `life-pixel-core` fit within the format's bounds.
pub fn export_wasm(animation: &Animation) -> Result<Vec<u8>, ExportError> {
    let payload = life_pixel_format::encode(&flatten::animation_data(animation))
        .map_err(ExportError::encoding)?;
    let mut module = PLAYER_WASM.to_vec();
    custom_section::append(&mut module, SECTION_NAME, &payload)?;
    Ok(module)
}
