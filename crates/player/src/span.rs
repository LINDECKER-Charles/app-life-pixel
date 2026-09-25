/// Where a part of the payload lies within the bytes `load` received: a frame's data, the title,
/// a tag's name. The player keeps spans rather than borrows, since it owns those bytes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct Span {
    start: usize,
    len: usize,
}

impl Span {
    /// Where `part`, a subslice of `payload`, lies in it.
    pub(crate) fn of(payload: &[u8], part: &[u8]) -> Self {
        let start = part.as_ptr().addr().wrapping_sub(payload.as_ptr().addr());
        Self {
            start,
            len: part.len(),
        }
    }

    /// The bytes of `payload` this span covers; empty if they are not there.
    pub(crate) fn bytes(self, payload: &[u8]) -> &[u8] {
        let end = self.start.saturating_add(self.len);
        payload.get(self.start..end).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_span_finds_its_part_again() {
        let payload = *b"LPIX title";
        let span = Span::of(&payload, &payload[5..]);

        assert_eq!(span.bytes(&payload), b"title");
    }

    #[test]
    fn a_span_outside_the_payload_is_empty() {
        let payload = [0; 4];
        let elsewhere = [0; 8];

        assert!(Span::of(&payload, &elsewhere).bytes(&payload).is_empty());
    }
}
