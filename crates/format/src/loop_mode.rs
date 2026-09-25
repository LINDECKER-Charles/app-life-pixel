/// How a tag plays when it reaches its last frame. Byte `0` is [`LoopMode::Loop`], byte `1`
/// [`LoopMode::Once`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LoopMode {
    /// Goes back to the tag's first frame.
    Loop,
    /// Stays on the tag's last frame and stops.
    Once,
}
