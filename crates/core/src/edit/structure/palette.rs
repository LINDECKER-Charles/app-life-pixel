use crate::edit::change::{CelChange, Change};
use crate::edit::references::editable_palette_index;
use crate::edit::{EditError, Operation};
use crate::limits::MAX_PALETTE_ENTRIES;
use crate::model::{Animation, Cel, Palette, Rgba};

/// A new index for every old one: what a palette change does to the cels.
type Remapping = [u8; MAX_PALETTE_ENTRIES];

pub(super) fn plan(
    animation: &Animation,
    operation: &Operation,
) -> Option<Result<Change, EditError>> {
    let change = match operation {
        Operation::SetPaletteEntry { index, color } => set_entry(animation, *index, *color),
        Operation::AddPaletteEntry { color } => add_entry(animation, *color),
        Operation::RemovePaletteEntry { index } => remove_entry(animation, *index),
        Operation::MovePaletteEntry { from, to } => move_entry(animation, *from, *to),
        _ => return None,
    };
    Some(change)
}

fn set_entry(animation: &Animation, index: u32, color: Rgba) -> Result<Change, EditError> {
    let index = editable_palette_index(animation, index)?;
    let mut entries = animation.palette().entries().to_vec();
    entries[usize::from(index)] = color;
    palette_change(entries, Vec::new())
}

fn add_entry(animation: &Animation, color: Rgba) -> Result<Change, EditError> {
    if animation.palette().len() >= MAX_PALETTE_ENTRIES {
        return Err(EditError::PaletteFull);
    }
    let mut entries = animation.palette().entries().to_vec();
    entries.push(color);
    palette_change(entries, Vec::new())
}

/// Its pixels become 0; higher indices move down by one.
fn remove_entry(animation: &Animation, index: u32) -> Result<Change, EditError> {
    let removed = editable_palette_index(animation, index)?;
    let mut entries = animation.palette().entries().to_vec();
    entries.remove(usize::from(removed));
    let remapping = remapping(|old| match old {
        _ if old == removed => 0,
        _ if old > removed => old - 1,
        _ => old,
    });
    palette_change(entries, remap(animation, &remapping))
}

/// The entries between both positions shift by one to make room.
fn move_entry(animation: &Animation, from: u32, to: u32) -> Result<Change, EditError> {
    let from = editable_palette_index(animation, from)?;
    let to = editable_palette_index(animation, to)?;
    let mut entries = animation.palette().entries().to_vec();
    let moved = entries.remove(usize::from(from));
    entries.insert(usize::from(to), moved);
    let remapping = remapping(|old| match old {
        _ if old == from => to,
        _ if from < old && old <= to => old - 1,
        _ if to <= old && old < from => old + 1,
        _ => old,
    });
    palette_change(entries, remap(animation, &remapping))
}

fn palette_change(entries: Vec<Rgba>, cels: Vec<CelChange>) -> Result<Change, EditError> {
    Ok(Change {
        palette: Some(Palette::new(entries)?),
        cels,
        ..Change::default()
    })
}

fn remapping(new_index: impl Fn(u8) -> u8) -> Remapping {
    let mut remapping = [0; MAX_PALETTE_ENTRIES];
    for (old, slot) in (0..=u8::MAX).zip(remapping.iter_mut()) {
        *slot = new_index(old);
    }
    remapping
}

/// Every cel the remapping changes, remapped.
fn remap(animation: &Animation, remapping: &Remapping) -> Vec<CelChange> {
    let mut changes = Vec::new();
    for (key, cel) in animation.cels() {
        let indices: Vec<u8> = cel
            .indices()
            .iter()
            .map(|&index| remapping[usize::from(index)])
            .collect();
        if indices != cel.indices() {
            changes.push((*key, Some(Cel::new(indices))));
        }
    }
    changes
}
