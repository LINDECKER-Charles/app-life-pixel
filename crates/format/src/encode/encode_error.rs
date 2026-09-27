use core::fmt;

/// Why an animation cannot be encoded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EncodeError {
    /// A size or count passes one of the [`bounds`](crate::bounds).
    BeyondBound,
    /// The animation contradicts itself: a frame of the wrong size, a tag out of range.
    Inconsistent,
}

impl fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BeyondBound => "animation beyond a format bound",
            Self::Inconsistent => "inconsistent animation",
        })
    }
}

impl core::error::Error for EncodeError {}
