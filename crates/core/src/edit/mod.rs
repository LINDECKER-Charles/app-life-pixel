//! The editing operations: the tools and edits of the editor as [`Operation`]s, each applied with
//! its [`Inverse`] for undo and redo, the [`History`] that keeps them, and the import of PNG images
//! and sprite sheets reduced to the palette.
//!
//! Every reference an operation makes is checked first; the rules of the model hold after every
//! operation, or it fails with the model's code and changes nothing.

mod change;
mod error;
mod history;
mod import;
mod inverse;
mod operation;
mod pixels;
mod references;
mod structure;

pub use error::EditError;
pub use history::History;
pub use inverse::Inverse;
pub use operation::{Area, Operation, TagSpec};

use change::Change;

use crate::model::{Animation, Cel, FrameId, LayerId};

/// Applies `operation` to `animation`, and returns what undoes it.
///
/// # Errors
///
/// The [`EditError`] of the first reference or rule the operation breaks; `animation` is then
/// left as it was.
pub fn apply(animation: &mut Animation, operation: &Operation) -> Result<Inverse, EditError> {
    let change = plan(animation, operation)?.without_unchanged(animation);
    let edited_cels: Option<Vec<(LayerId, FrameId)>> =
        (!change.touches_every_cel()).then(|| change.cels.iter().map(|(key, _)| *key).collect());
    let replaced = change.swap_into(animation);
    if let Err(error) = animation.check_edited(edited_cels.as_deref()) {
        let _broken = replaced.swap_into(animation);
        return Err(error.into());
    }
    Ok(Inverse::new(replaced))
}

/// The cel a pixel operation would produce on its layer and frame, without changing the
/// animation — what the canvas draws while a stroke, a line or a rectangle is in progress. `None`
/// for any other operation.
///
/// # Errors
///
/// The [`EditError`] [`apply`] would fail with, for a reference or limit the operation breaks.
pub fn preview(
    animation: &Animation,
    operation: &Operation,
) -> Result<Option<(LayerId, FrameId, Cel)>, EditError> {
    pixels::plan(animation, operation).transpose()
}

/// The change `operation` makes to `animation`, before the model's rules are checked.
fn plan(animation: &Animation, operation: &Operation) -> Result<Change, EditError> {
    let edited_cel = pixels::plan(animation, operation)
        .map(|edited| edited.map(|(layer, frame, cel)| Change::cel((layer, frame), cel)));
    edited_cel
        .or_else(|| structure::plan(animation, operation))
        .or_else(|| import::plan_sheet(animation, operation))
        .unwrap_or_else(|| Ok(Change::default()))
}
