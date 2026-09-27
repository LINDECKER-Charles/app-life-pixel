use core::fmt;

/// Why a payload is refused. Each variant maps to a status of player ABI v1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DecodeError {
    /// The payload does not start with [`MAGIC`](crate::MAGIC). Status 1.
    BadMagic,
    /// The format version is not [`FORMAT_VERSION`](crate::FORMAT_VERSION). Status 2.
    UnknownFormatVersion,
    /// The ABI version is not [`ABI_VERSION`](crate::ABI_VERSION). Status 3.
    UnknownAbiVersion,
    /// The bytes break a rule of payload v1: truncated, trailing or out-of-range. Status 4.
    Malformed,
    /// A size or count passes one of the [`bounds`](crate::bounds). Status 5.
    BeyondBound,
}

impl DecodeError {
    /// The status the player returns for this error, as player ABI v1 numbers them: 1 to 5.
    #[must_use]
    pub fn status(&self) -> u32 {
        match self {
            Self::BadMagic => 1,
            Self::UnknownFormatVersion => 2,
            Self::UnknownAbiVersion => 3,
            Self::Malformed => 4,
            Self::BeyondBound => 5,
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::BadMagic => "not a Life Pixel payload",
            Self::UnknownFormatVersion => "unknown payload format version",
            Self::UnknownAbiVersion => "unknown player ABI version",
            Self::Malformed => "malformed payload",
            Self::BeyondBound => "payload beyond a decoder bound",
        })
    }
}

impl core::error::Error for DecodeError {}

#[cfg(test)]
mod tests {
    use super::DecodeError;

    #[test]
    fn statuses_follow_player_abi_v1() {
        let statuses = [
            DecodeError::BadMagic,
            DecodeError::UnknownFormatVersion,
            DecodeError::UnknownAbiVersion,
            DecodeError::Malformed,
            DecodeError::BeyondBound,
        ]
        .map(|error| error.status());

        assert_eq!(statuses, [1, 2, 3, 4, 5]);
    }
}
