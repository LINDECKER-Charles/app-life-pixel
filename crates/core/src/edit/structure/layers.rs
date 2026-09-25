use super::new_ids;
use crate::edit::change::Change;
use crate::edit::references::{layer_position, position};
use crate::edit::{EditError, Operation};
use crate::model::{Animation, Layer, LayerId, Name};

pub(super) fn plan(
    animation: &Animation,
    operation: &Operation,
) -> Option<Result<Change, EditError>> {
    let change = match operation {
        Operation::AddLayer { position, name } => add(animation, *position, name),
        Operation::DeleteLayer { layer } => delete(animation, *layer),
        Operation::MoveLayer { layer, position } => move_to(animation, *layer, *position),
        Operation::RenameLayer { layer, name } => edit(animation, *layer, |layer| {
            let renamed = Layer::new(layer.id(), Name::new(name)?);
            Ok(renamed.with_visibility(layer.is_visible()))
        }),
        Operation::SetLayerVisibility { layer, visible } => edit(animation, *layer, |layer| {
            Ok(layer.clone().with_visibility(*visible))
        }),
        _ => return None,
    };
    Some(change)
}

fn add(animation: &Animation, at: u32, name: &str) -> Result<Change, EditError> {
    let mut layers = animation.layers().to_vec();
    let at = position(at, layers.len())?;
    let name = Name::new(name)?;
    let ids = new_ids(animation, 1)?;
    layers.insert(at, Layer::new(LayerId::new(ids.start), name));
    Ok(Change {
        layers: Some(layers),
        next_id: Some(ids.end),
        ..Change::default()
    })
}

/// Its cels go too.
fn delete(animation: &Animation, layer: LayerId) -> Result<Change, EditError> {
    let at = layer_position(animation, layer)?;
    let mut layers = animation.layers().to_vec();
    if layers.len() == 1 {
        return Err(EditError::LastLayer);
    }
    layers.remove(at);
    let cels = animation.cels().keys().filter(|(owner, _)| *owner == layer);
    Ok(Change {
        layers: Some(layers),
        cels: cels.map(|key| (*key, None)).collect(),
        ..Change::default()
    })
}

fn move_to(animation: &Animation, layer: LayerId, to: u32) -> Result<Change, EditError> {
    let from = layer_position(animation, layer)?;
    let mut layers = animation.layers().to_vec();
    let to = position(to, layers.len() - 1)?;
    let moved = layers.remove(from);
    layers.insert(to, moved);
    Ok(Change {
        layers: Some(layers),
        ..Change::default()
    })
}

/// Replaces the layer `id` by what `change` makes of it.
fn edit(
    animation: &Animation,
    id: LayerId,
    change: impl FnOnce(&Layer) -> Result<Layer, EditError>,
) -> Result<Change, EditError> {
    let at = layer_position(animation, id)?;
    let mut layers = animation.layers().to_vec();
    layers[at] = change(&layers[at])?;
    Ok(Change {
        layers: Some(layers),
        ..Change::default()
    })
}
