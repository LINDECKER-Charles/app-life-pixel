use serde::{Deserialize, Serialize};

use crate::error::DocumentError;
use crate::model::{LoopMode, Tag, TagName};

/// A tag as an operation gives it, in the shape of a document's tag: checked when applied.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TagSpec {
    /// The tag's name.
    pub name: String,
    /// The position of its first frame, from 0.
    pub first: u32,
    /// The position of its last frame, included.
    pub last: u32,
    /// How it plays at its end.
    #[serde(rename = "loop")]
    pub loop_mode: LoopMode,
}

impl TagSpec {
    /// The tag this spec describes. Whether it fits the frames and its name is unique is the
    /// animation's rule.
    ///
    /// # Errors
    ///
    /// [`DocumentError::Tag`] when the name breaks a rule of tag names or a position is out of
    /// range.
    pub fn to_tag(&self) -> Result<Tag, DocumentError> {
        let name = TagName::new(&self.name)?;
        let out_of_range = || DocumentError::Tag {
            name: self.name.clone(),
        };
        let first = u16::try_from(self.first).map_err(|_| out_of_range())?;
        let last = u16::try_from(self.last).map_err(|_| out_of_range())?;
        Ok(Tag::new(name, first..=last, self.loop_mode))
    }
}

impl From<&Tag> for TagSpec {
    fn from(tag: &Tag) -> Self {
        Self {
            name: tag.name().to_string(),
            first: u32::from(tag.first()),
            last: u32::from(tag.last()),
            loop_mode: tag.loop_mode(),
        }
    }
}
