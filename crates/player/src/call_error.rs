use life_pixel_format::DecodeError;

/// Why a call of player ABI v1 did not complete. [`CallError::status`] is what the export
/// returns; `0`, done, is a call that completed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallError {
    /// `load` refused the payload: statuses 1 to 5, as the decoder numbers them.
    Payload(DecodeError),
    /// `load` could not reserve what the payload needs: status 6.
    OutOfMemory,
    /// A call before a successful `load`, or a second `alloc` or `load`: status 7.
    WrongCallOrder,
    /// An argument outside what the call accepts: status 8.
    ArgumentOutOfRange,
}

impl CallError {
    /// The status the export returns for this error, as player ABI v1 numbers them: 1 to 8.
    #[must_use]
    pub fn status(&self) -> u32 {
        match self {
            Self::Payload(error) => error.status(),
            Self::OutOfMemory => 6,
            Self::WrongCallOrder => 7,
            Self::ArgumentOutOfRange => 8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_follow_player_abi_v1() {
        let statuses = [
            CallError::Payload(DecodeError::BadMagic),
            CallError::Payload(DecodeError::BeyondBound),
            CallError::OutOfMemory,
            CallError::WrongCallOrder,
            CallError::ArgumentOutOfRange,
        ]
        .map(|error| error.status());

        assert_eq!(statuses, [1, 5, 6, 7, 8]);
    }
}
