//! What a document file says — its title and size, and its version —, cached by path,
//! modification time and length so that a list parses only what changed.

use std::collections::HashMap;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use std::time::SystemTime;

use life_pixel_core::serialize::read_document;
use sha2::{Digest as _, Sha256};

use crate::ports::library_store::AnimationMeta;

/// How many bytes of the SHA-256 make a version: 48 bits, below 2^53, so that a JavaScript
/// number carries it exactly.
const VERSION_BYTES: usize = 6;

/// What lists show of a document, and its version.
#[derive(Clone, Debug)]
pub(super) struct Summary {
    pub(super) meta: AnimationMeta,
    pub(super) version: u64,
}

/// A summary, and the file it was read from.
struct Entry {
    modified: SystemTime,
    length: u64,
    summary: Summary,
}

/// The summaries of the documents already read, by path.
#[derive(Default)]
pub(super) struct DocumentCache {
    entries: Mutex<HashMap<PathBuf, Entry>>,
}

impl DocumentCache {
    /// The summary of the file at `path`, if it has not changed since it was read.
    pub(super) fn get(&self, path: &Path, metadata: &Metadata) -> Option<Summary> {
        let entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        let entry = entries.get(path)?;
        let is_same_file =
            metadata.modified().ok() == Some(entry.modified) && metadata.len() == entry.length;
        is_same_file.then(|| entry.summary.clone())
    }

    /// Keeps the summary of the file at `path`, as its metadata describes it.
    pub(super) fn insert(&self, path: &Path, (metadata, summary): (&Metadata, Summary)) {
        let Ok(modified) = metadata.modified() else {
            return;
        };
        let entry = Entry {
            modified,
            length: metadata.len(),
            summary,
        };
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        entries.insert(path.to_path_buf(), entry);
    }

    /// Forgets the files under `path`.
    pub(super) fn forget(&self, path: &Path) {
        let mut entries = self.entries.lock().unwrap_or_else(PoisonError::into_inner);
        entries.retain(|cached, _| !cached.starts_with(path));
    }
}

/// The summary of the document `bytes`, or `None` when `core` cannot read it.
pub(super) fn summarize(bytes: &[u8]) -> Option<Summary> {
    let animation = read_document(bytes).ok()?;
    let meta = AnimationMeta {
        title: animation.title().clone(),
        width: animation.width(),
        height: animation.height(),
        frame_count: u16::try_from(animation.frames().len()).unwrap_or(u16::MAX),
    };
    let version = version_of(bytes);
    Some(Summary { meta, version })
}

/// The version of a file holding `bytes`: the first 6 bytes of their SHA-256, as an integer.
pub(super) fn version_of(bytes: &[u8]) -> u64 {
    let digest = Sha256::digest(bytes);
    let mut version = [0; size_of::<u64>()];
    version[size_of::<u64>() - VERSION_BYTES..].copy_from_slice(&digest[..VERSION_BYTES]);
    u64::from_be_bytes(version)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_version_is_the_start_of_the_sha256_below_2_pow_53() {
        // SHA-256("abc") = ba7816bf8f01cfea…
        assert_eq!(version_of(b"abc"), 0xba78_16bf_8f01);
        assert!(version_of(b"abc") < 1 << 53);
        assert_ne!(version_of(b"abc"), version_of(b"abd"));
    }
}
