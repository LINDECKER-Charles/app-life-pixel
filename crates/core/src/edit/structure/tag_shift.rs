//! What inserting or deleting frames does to the tags, which name frames by position.

use crate::model::Tag;

/// The tags once `count` frames are inserted at `position`: a tag starting at or after it moves
/// later, a tag spanning it grows.
pub(crate) fn tags_after_insert(tags: &[Tag], position: usize, count: usize) -> Vec<Tag> {
    let position = narrow(position);
    let count = narrow(count);
    let shifted = |tag: &Tag| {
        let (first, last) = (tag.first(), tag.last());
        if first >= position {
            (first.saturating_add(count), last.saturating_add(count))
        } else if last >= position {
            (first, last.saturating_add(count))
        } else {
            (first, last)
        }
    };
    tags.iter()
        .map(|tag| with_range(tag, shifted(tag)))
        .collect()
}

/// The tags once the frame at `position` is deleted: a tag after it moves earlier, a tag
/// spanning it shrinks, and a tag of that frame alone goes.
pub(crate) fn tags_after_delete(tags: &[Tag], position: usize) -> Vec<Tag> {
    let position = narrow(position);
    let shifted = |tag: &Tag| {
        let (first, last) = (tag.first(), tag.last());
        if first == position && last == position {
            return None;
        }
        let first = if first > position { first - 1 } else { first };
        let last = if last >= position { last - 1 } else { last };
        Some((first, last))
    };
    tags.iter()
        .filter_map(|tag| shifted(tag).map(|range| with_range(tag, range)))
        .collect()
}

fn with_range(tag: &Tag, (first, last): (u16, u16)) -> Tag {
    Tag::new(tag.name().clone(), first..=last, tag.loop_mode())
}

/// A frame position or count in the width tags hold it; positions never reach that far, as an
/// animation holds at most `MAX_FRAMES` frames.
fn narrow(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}
