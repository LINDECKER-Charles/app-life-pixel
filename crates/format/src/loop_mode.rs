/// How a tag plays when it reaches its last frame. Byte `0` is [`LoopMode::Loop`], byte `1`
/// [`LoopMode::Once`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoopMode {
    /// Goes back to the tag's first frame.
    Loop,
    /// Stays on the tag's last frame and stops.
    Once,
}

impl LoopMode {
    /// The mode byte `byte` stands for, or `None` above 1.
    pub(crate) fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Loop),
            1 => Some(Self::Once),
            _ => None,
        }
    }

    /// The byte that stands for this mode.
    #[cfg(feature = "encode")]
    pub(crate) fn to_byte(self) -> u8 {
        match self {
            Self::Loop => 0,
            Self::Once => 1,
        }
    }
}
