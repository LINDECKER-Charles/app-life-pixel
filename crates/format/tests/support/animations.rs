use life_pixel_format::{AnimationData, FrameData, LoopMode, Rgba, TagData};

/// `len` palette entries: entry 0 transparent, the others opaque.
pub fn palette(len: usize) -> Vec<Rgba> {
    let opaque = |value: u8| Rgba {
        r: value,
        g: value.wrapping_mul(7),
        b: 0xFF - value,
        a: 0xFF,
    };
    (0..len)
        .map(|entry| match entry {
            0 => Rgba::default(),
            _ => opaque(entry as u8),
        })
        .collect()
}

/// An animation of `frames`, 100 ms each, without title or tags, over just enough palette
/// entries for its largest index.
pub fn animation(width: u16, height: u16, frames: Vec<Vec<u8>>) -> AnimationData {
    let largest_index = frames.iter().flatten().max().copied().unwrap_or_default();
    AnimationData {
        width,
        height,
        palette: palette(usize::from(largest_index) + 1),
        title: String::new(),
        tags: Vec::new(),
        frames: frames
            .into_iter()
            .map(|indices| FrameData {
                duration_ms: 100,
                indices,
            })
            .collect(),
    }
}

/// A tag named `name` over `first..=last`, looping.
pub fn tag(name: &str, first: u16, last: u16) -> TagData {
    TagData {
        name: name.to_owned(),
        first,
        last,
        loop_mode: LoopMode::Loop,
    }
}

/// `len` indices below `palette_len`, deterministic for a `seed`, mixing runs and noise.
pub fn pattern(len: usize, palette_len: usize, seed: u32) -> Vec<u8> {
    let mut state = seed.wrapping_mul(2_654_435_761).max(1);
    let mut indices = Vec::with_capacity(len);
    while indices.len() < len {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        let index = (state as usize % palette_len) as u8;
        let repeat = 1 + (state >> 24) as usize % 6;
        indices.extend(std::iter::repeat_n(index, repeat.min(len - indices.len())));
    }
    indices
}
