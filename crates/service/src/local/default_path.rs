//! Where the library lives when the user has not chosen a folder.

use std::path::PathBuf;

/// The name of the default library folder.
const LIBRARY_FOLDER: &str = "Life Pixel";

/// The default library folder: `<documents>/Life Pixel`, or `<home>/Life Pixel` without a
/// documents folder. The CLI and the desktop app pass what `dirs` finds.
#[must_use]
pub fn default_library_path(documents: Option<PathBuf>, home: PathBuf) -> PathBuf {
    documents.unwrap_or(home).join(LIBRARY_FOLDER)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_library_is_in_the_documents_folder() {
        let path = default_library_path(Some("/Users/ada/Documents".into()), "/Users/ada".into());
        assert_eq!(path, PathBuf::from("/Users/ada/Documents/Life Pixel"));
    }

    #[test]
    fn without_a_documents_folder_the_library_is_in_home() {
        let path = default_library_path(None, "/home/ada".into());
        assert_eq!(path, PathBuf::from("/home/ada/Life Pixel"));
    }
}
