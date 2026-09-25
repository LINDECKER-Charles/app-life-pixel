use crate::edit::change::Change;
use crate::edit::references::tag_position;
use crate::edit::{EditError, Operation, TagSpec};
use crate::model::{Animation, Tag};

/// Tags are found by name; whether they fit the frames and keep unique names is the model's rule.
pub(super) fn plan(
    animation: &Animation,
    operation: &Operation,
) -> Option<Result<Change, EditError>> {
    let tags = match operation {
        Operation::AddTag { tag } => add(animation, tag),
        Operation::UpdateTag { name, tag } => update(animation, name, tag),
        Operation::DeleteTag { name } => delete(animation, name),
        Operation::ReplaceTags { tags } => tags
            .iter()
            .map(|tag| tag.to_tag().map_err(EditError::from))
            .collect(),
        _ => return None,
    };
    let change = tags.map(|tags| Change {
        tags: Some(tags),
        ..Change::default()
    });
    Some(change)
}

fn add(animation: &Animation, spec: &TagSpec) -> Result<Vec<Tag>, EditError> {
    let mut tags = animation.tags().to_vec();
    tags.push(spec.to_tag()?);
    Ok(tags)
}

fn update(animation: &Animation, name: &str, spec: &TagSpec) -> Result<Vec<Tag>, EditError> {
    let at = tag_position(animation, name)?;
    let mut tags = animation.tags().to_vec();
    tags[at] = spec.to_tag()?;
    Ok(tags)
}

fn delete(animation: &Animation, name: &str) -> Result<Vec<Tag>, EditError> {
    let at = tag_position(animation, name)?;
    let mut tags = animation.tags().to_vec();
    tags.remove(at);
    Ok(tags)
}
