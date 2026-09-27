//! A frame as pixels: its composite of palette indices, then its RGBA. The editor, the exports
//! and the MCP previews all go through here — what the editor shows is what ships.

mod composite;
mod rgba;

pub use composite::{Replacement, composite, composite_with};
pub use rgba::{rgba, rgba_with};
