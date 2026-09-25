//! A change: new values for the parts of an animation an operation touches. Swapping one into an
//! animation returns the values it replaced — another change, which swaps them back.

use std::mem;

use crate::model::{Animation, Cel, Frame, FrameId, Layer, LayerId, Name, Palette, Tag};

/// The bytes a change is counted for beyond its cels and names: its fixed part.
const FIXED_BYTES: usize = mem::size_of::<Change>();
/// The bytes counted for one layer, one frame or one tag beyond its name.
const ITEM_BYTES: usize = 16;
/// The bytes of one palette entry.
const ENTRY_BYTES: usize = 4;

/// The cel of a key: `None` or a blank cel removes it.
pub(crate) type CelChange = ((LayerId, FrameId), Option<Cel>);

/// New values for the parts of an animation it names; `None` leaves a part as it is.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Change {
    pub(crate) title: Option<Name>,
    pub(crate) palette: Option<Palette>,
    pub(crate) layers: Option<Vec<Layer>>,
    pub(crate) frames: Option<Vec<Frame>>,
    pub(crate) tags: Option<Vec<Tag>>,
    pub(crate) next_id: Option<u32>,
    /// One entry per key at most.
    pub(crate) cels: Vec<CelChange>,
}

impl Change {
    /// The change of one cel.
    pub(crate) fn cel(key: (LayerId, FrameId), cel: Cel) -> Self {
        Self {
            cels: vec![(key, Some(cel))],
            ..Self::default()
        }
    }

    /// Whether it changes nothing.
    pub(crate) fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Whether it replaces the palette, the layers or the frames: what every cel depends on.
    pub(crate) fn touches_every_cel(&self) -> bool {
        self.palette.is_some() || self.layers.is_some() || self.frames.is_some()
    }

    /// The same change without the values `animation` already has.
    pub(crate) fn without_unchanged(self, animation: &Animation) -> Self {
        let cels = self
            .cels
            .into_iter()
            .filter(|(key, cel)| {
                let new = cel.as_ref().filter(|cel| !cel.is_blank());
                new != animation.cels.get(key)
            })
            .collect();
        Self {
            title: self.title.filter(|title| *title != animation.title),
            palette: self.palette.filter(|palette| *palette != animation.palette),
            layers: self.layers.filter(|layers| *layers != animation.layers),
            frames: self.frames.filter(|frames| *frames != animation.frames),
            tags: self.tags.filter(|tags| *tags != animation.tags),
            next_id: self.next_id.filter(|next_id| *next_id != animation.next_id),
            cels,
        }
    }

    /// Puts its values into `animation` and returns those it replaced.
    pub(crate) fn swap_into(self, animation: &mut Animation) -> Self {
        let mut cels: Vec<CelChange> = self
            .cels
            .into_iter()
            .map(|(key, cel)| (key, swap_cel(animation, key, cel)))
            .collect();
        cels.reverse();
        Self {
            title: swap(&mut animation.title, self.title),
            palette: swap(&mut animation.palette, self.palette),
            layers: swap(&mut animation.layers, self.layers),
            frames: swap(&mut animation.frames, self.frames),
            tags: swap(&mut animation.tags, self.tags),
            next_id: swap(&mut animation.next_id, self.next_id),
            cels,
        }
    }

    /// Roughly the memory it holds, in bytes: what the history counts.
    pub(crate) fn byte_size(&self) -> usize {
        let cels: usize = self
            .cels
            .iter()
            .map(|(_, cel)| cel_bytes(cel.as_ref()))
            .sum();
        let title = self.title.as_ref().map_or(0, |title| title.as_str().len());
        let palette = self
            .palette
            .as_ref()
            .map_or(0, |palette| palette.len() * ENTRY_BYTES);
        let layers = self.layers.as_ref().map_or(0, |layers| {
            layers
                .iter()
                .map(|layer| ITEM_BYTES + layer.name().as_str().len())
                .sum()
        });
        let frames = self
            .frames
            .as_ref()
            .map_or(0, |frames| frames.len() * ITEM_BYTES);
        let tags = self.tags.as_ref().map_or(0, |tags| {
            tags.iter()
                .map(|tag| ITEM_BYTES + tag.name().as_str().len())
                .sum()
        });
        FIXED_BYTES + cels + title + palette + layers + frames + tags
    }
}

fn swap<T>(part: &mut T, new: Option<T>) -> Option<T> {
    new.map(|value| mem::replace(part, value))
}

fn swap_cel(animation: &mut Animation, key: (LayerId, FrameId), cel: Option<Cel>) -> Option<Cel> {
    match cel.filter(|cel| !cel.is_blank()) {
        Some(cel) => animation.cels.insert(key, cel),
        None => animation.cels.remove(&key),
    }
}

fn cel_bytes(cel: Option<&Cel>) -> usize {
    ITEM_BYTES + cel.map_or(0, |cel| cel.indices().len())
}
