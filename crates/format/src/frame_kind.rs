/// Whether a frame stands alone or builds on the previous one. Byte `0` is [`FrameKind::Key`],
/// byte `1` [`FrameKind::Delta`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameKind {
    /// Sets every pixel; SKIP operations are refused.
    Key,
    /// Applies to the previous frame's indices; SKIP keeps them.
    Delta,
}

impl FrameKind {
    /// The kind byte `byte` stands for, or `None` above 1.
    pub(crate) fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::Key),
            1 => Some(Self::Delta),
            _ => None,
        }
    }

    /// The byte that stands for this kind.
    #[cfg(feature = "encode")]
    pub(crate) fn to_byte(self) -> u8 {
        match self {
            Self::Key => 0,
            Self::Delta => 1,
        }
    }
}
