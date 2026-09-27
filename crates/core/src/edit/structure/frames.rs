use super::new_ids;
use super::tag_shift::{tags_after_delete, tags_after_insert};
use crate::edit::change::Change;
use crate::edit::references::{frame_duration, frame_position, position};
use crate::edit::{EditError, Operation};
use crate::model::{Animation, Frame, FrameId};

pub(super) fn plan(
    animation: &Animation,
    operation: &Operation,
) -> Option<Result<Change, EditError>> {
    let change = match operation {
        Operation::AddFrame {
            position,
            duration_ms,
        } => add(animation, *position, *duration_ms),
        Operation::DuplicateFrame { frame } => duplicate(animation, *frame),
        Operation::DeleteFrame { frame } => delete(animation, *frame),
        Operation::MoveFrame { frame, position } => move_to(animation, *frame, *position),
        Operation::SetFrameDuration { frame, duration_ms } => {
            set_duration(animation, *frame, *duration_ms)
        }
        _ => return None,
    };
    Some(change)
}

/// Its cels are blank.
fn add(animation: &Animation, at: u32, duration_ms: u32) -> Result<Change, EditError> {
    let at = position(at, animation.frames().len())?;
    let duration_ms = frame_duration(duration_ms)?;
    let ids = new_ids(animation, 1)?;
    let frame = Frame::new(FrameId::new(ids.start), duration_ms)?;
    let mut change = inserted(animation, at, frame);
    change.next_id = Some(ids.end);
    Ok(change)
}

/// The copy, cels included, goes right after it.
fn duplicate(animation: &Animation, frame: FrameId) -> Result<Change, EditError> {
    let at = frame_position(animation, frame)?;
    let ids = new_ids(animation, 1)?;
    let copy = Frame::new(
        FrameId::new(ids.start),
        animation.frames()[at].duration_ms(),
    )?;
    let mut change = inserted(animation, at + 1, copy);
    change.next_id = Some(ids.end);
    change.cels = animation
        .layers()
        .iter()
        .filter_map(|layer| {
            let cel = animation.cel(layer.id(), frame)?;
            Some(((layer.id(), copy.id()), Some(cel.clone())))
        })
        .collect();
    Ok(change)
}

/// Its cels go too.
fn delete(animation: &Animation, frame: FrameId) -> Result<Change, EditError> {
    let at = frame_position(animation, frame)?;
    let mut frames = animation.frames().to_vec();
    if frames.len() == 1 {
        return Err(EditError::LastFrame);
    }
    frames.remove(at);
    let cels = animation.cels().keys().filter(|(_, owner)| *owner == frame);
    Ok(Change {
        frames: Some(frames),
        tags: Some(tags_after_delete(animation.tags(), at)),
        cels: cels.map(|key| (*key, None)).collect(),
        ..Change::default()
    })
}

/// Tags keep their positions.
fn move_to(animation: &Animation, frame: FrameId, to: u32) -> Result<Change, EditError> {
    let from = frame_position(animation, frame)?;
    let mut frames = animation.frames().to_vec();
    let to = position(to, frames.len() - 1)?;
    let moved = frames.remove(from);
    frames.insert(to, moved);
    Ok(Change {
        frames: Some(frames),
        ..Change::default()
    })
}

fn set_duration(
    animation: &Animation,
    frame: FrameId,
    duration_ms: u32,
) -> Result<Change, EditError> {
    let at = frame_position(animation, frame)?;
    let duration_ms = frame_duration(duration_ms)?;
    let mut frames = animation.frames().to_vec();
    frames[at] = Frame::new(frame, duration_ms)?;
    Ok(Change {
        frames: Some(frames),
        ..Change::default()
    })
}

/// The frames and tags once `frame` is inserted at `at`.
fn inserted(animation: &Animation, at: usize, frame: Frame) -> Change {
    let mut frames = animation.frames().to_vec();
    frames.insert(at, frame);
    Change {
        frames: Some(frames),
        tags: Some(tags_after_insert(animation.tags(), at, 1)),
        ..Change::default()
    }
}
