/// Whether a frame stands alone or builds on the previous one. Byte `0` is [`FrameKind::Key`],
/// byte `1` [`FrameKind::Delta`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FrameKind {
    /// Sets every pixel; SKIP operations are refused.
    Key,
    /// Applies to the previous frame's indices; SKIP keeps them.
    Delta,
}
