//! A whole palette as `core`'s palette operations: entries dropped from the top, changed in
//! place, then appended.

use life_pixel_core::edit::Operation;
use life_pixel_core::{Animation, Palette, Rgba};

use crate::animation::EditingError;
use crate::animation::addressing::apply_all;

/// Gives `animation` the palette `colors`.
pub(super) fn set_palette(animation: &mut Animation, colors: &[Rgba]) -> Result<(), EditingError> {
    let _checked = Palette::new(colors.to_vec())?;
    if let Some(index) = first_used_from(animation, colors.len()) {
        return Err(EditingError::PaletteInUse { index });
    }
    let operations = operations(animation.palette().entries(), colors);
    Ok(apply_all(animation, &operations)?)
}

/// The lowest palette index at or above `kept` that a cel paints, hidden layers included.
fn first_used_from(animation: &Animation, kept: usize) -> Option<usize> {
    let cels = animation.cels().values();
    let indices = cels.flat_map(|cel| cel.indices().iter().copied().map(usize::from));
    indices.filter(|&index| index >= kept).min()
}

/// The operations from `current` to `colors`: entry 0 is never touched.
fn operations(current: &[Rgba], colors: &[Rgba]) -> Vec<Operation> {
    let removed = (colors.len()..current.len())
        .rev()
        .map(|index| Operation::RemovePaletteEntry {
            index: entry(index),
        });
    let changed = (1..colors.len().min(current.len()))
        .filter(|&index| current[index] != colors[index])
        .map(|index| Operation::SetPaletteEntry {
            index: entry(index),
            color: colors[index],
        });
    let added = colors
        .iter()
        .skip(current.len())
        .map(|&color| Operation::AddPaletteEntry { color });
    removed.chain(changed).chain(added).collect()
}

/// A palette position as an operation's index; a palette never holds more than 256 entries.
fn entry(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}
