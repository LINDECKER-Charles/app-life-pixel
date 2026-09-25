//! Where documents are stored: `documents/<account>/<animation>/<uuid>.json`, a new key per
//! write, so that a write never overwrites the object a row still points at.

use life_pixel_service::{AccountId, AnimationId};
use object_store::path::Path;
use uuid::Uuid;

/// The prefix of every document.
pub const DOCUMENTS_PREFIX: &str = "documents";
/// The extension of a document's key.
const DOCUMENT_EXTENSION: &str = "json";

/// A new key for a document of `animation`.
#[must_use]
pub fn new_document_key(account: AccountId, animation: AnimationId) -> Path {
    let name = format!("{}.{DOCUMENT_EXTENSION}", Uuid::now_v7());
    animation_prefix(account, animation).join(name)
}

/// The prefix of every document of `animation`, those of its past writes included.
#[must_use]
pub fn animation_prefix(account: AccountId, animation: AnimationId) -> Path {
    Path::from_iter([
        DOCUMENTS_PREFIX.to_owned(),
        account.to_string(),
        animation.to_string(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_document_key_is_new_on_each_write_and_under_its_animation() {
        let account = AccountId::from_uuid(Uuid::from_u128(1));
        let animation = AnimationId::from_uuid(Uuid::from_u128(2));

        let first = new_document_key(account, animation);
        let second = new_document_key(account, animation);

        assert_ne!(first, second);
        let prefix = "documents/00000000-0000-0000-0000-000000000001/\
                      00000000-0000-0000-0000-000000000002/";
        assert!(first.as_ref().starts_with(prefix), "{first}");
        assert!(first.as_ref().ends_with(".json"), "{first}");
    }
}
