//! One file an export produces.

/// A file ready to save or download: its name, its media type and its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportFile {
    /// The file name, from [`file_stem`](crate::file_stem): `mascot.gif`, for example.
    pub name: String,
    /// The media type: `image/gif`, for example.
    pub media_type: &'static str,
    /// The file's content.
    pub bytes: Vec<u8>,
}
