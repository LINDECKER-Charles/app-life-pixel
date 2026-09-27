//! The operations on the parts of an animation other than pixels: palette, layers, frames, tags
//! and title. Each gives the new values of the parts it changes.

mod frames;
mod layers;
mod palette;
mod tag_shift;
mod tags;

use std::ops::Range;

pub(crate) use tag_shift::tags_after_insert;

use super::change::Change;
use super::{EditError, Operation};
use crate::error::DocumentError;
use crate::model::{Animation, Name};

/// The change `operation` makes, when it is one of these operations.
pub(crate) fn plan(
    animation: &Animation,
    operation: &Operation,
) -> Option<Result<Change, EditError>> {
    palette::plan(animation, operation)
        .or_else(|| layers::plan(animation, operation))
        .or_else(|| frames::plan(animation, operation))
        .or_else(|| tags::plan(animation, operation))
        .or_else(|| set_title(operation))
}

/// The ids of `count` new layers or frames; `nextId` becomes the end of the range.
pub(crate) fn new_ids(animation: &Animation, count: usize) -> Result<Range<u32>, EditError> {
    let first = animation.next_id();
    let end = u32::try_from(count)
        .ok()
        .and_then(|count| first.checked_add(count))
        .ok_or(EditError::Document(DocumentError::Reference))?;
    Ok(first..end)
}

fn set_title(operation: &Operation) -> Option<Result<Change, EditError>> {
    let Operation::SetTitle { title } = operation else {
        return None;
    };
    let change = Name::new(title).map(|title| Change {
        title: Some(title),
        ..Change::default()
    });
    Some(change.map_err(EditError::from))
}
