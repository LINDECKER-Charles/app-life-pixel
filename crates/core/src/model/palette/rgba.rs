use std::fmt;
use std::str::FromStr;

use serde::{Serialize, Serializer};

use crate::error::DocumentError;

/// The prefix of a colour's text.
const HEX_PREFIX: char = '#';
/// The hexadecimal digits of a colour's text: two for each of red, green, blue and alpha.
const HEX_DIGITS: usize = 8;
/// The radix of a colour's digits.
const HEX_RADIX: u32 = 16;

/// A colour: red, green, blue and alpha, alpha not premultiplied. Written `#rrggbbaa`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Rgba {
    /// Red, 0 to 255.
    pub r: u8,
    /// Green, 0 to 255.
    pub g: u8,
    /// Blue, 0 to 255.
    pub b: u8,
    /// Alpha, 0 (transparent) to 255 (opaque).
    pub a: u8,
}

impl Rgba {
    /// The colour whose bytes are those of `value`, red first: `0xff0000ff` is opaque red.
    #[must_use]
    pub const fn from_u32(value: u32) -> Self {
        let [r, g, b, a] = value.to_be_bytes();
        Self { r, g, b, a }
    }

    /// The colour's four bytes, red first — how a renderer lays it out.
    #[must_use]
    pub const fn to_bytes(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }

    /// Whether the colour is fully transparent.
    #[must_use]
    pub const fn is_transparent(self) -> bool {
        self.a == 0
    }
}

impl FromStr for Rgba {
    type Err = DocumentError;

    /// Reads `#rrggbbaa`, digits in either case.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let digits = text
            .strip_prefix(HEX_PREFIX)
            .ok_or(DocumentError::Palette)?;
        let is_hex = digits.len() == HEX_DIGITS && digits.chars().all(|c| c.is_ascii_hexdigit());
        if !is_hex {
            return Err(DocumentError::Palette);
        }
        u32::from_str_radix(digits, HEX_RADIX)
            .map(Self::from_u32)
            .map_err(|_| DocumentError::Palette)
    }
}

impl fmt::Display for Rgba {
    /// Writes `#rrggbbaa`, in lowercase.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { r, g, b, a } = self;
        write!(formatter, "{HEX_PREFIX}{r:02x}{g:02x}{b:02x}{a:02x}")
    }
}

impl Serialize for Rgba {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_colour_reads_and_writes_as_rrggbbaa() {
        let colour: Rgba = "#FF7f27C0".parse().unwrap();
        assert_eq!(colour, Rgba::from_u32(0xff7f_27c0));
        assert_eq!(colour.to_string(), "#ff7f27c0");
    }

    #[test]
    fn a_colour_not_rrggbbaa_is_refused() {
        for text in [
            "ff7f27c0",
            "#ff7f27",
            "#ff7f27c0a",
            "#gg7f27c0",
            "#+f7f27c0",
            "#ff7f27é",
        ] {
            assert_eq!(
                text.parse::<Rgba>(),
                Err(DocumentError::Palette),
                "{text:?}"
            );
        }
    }
}
