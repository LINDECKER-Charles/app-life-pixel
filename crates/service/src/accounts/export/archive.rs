//! The zip of an export, written into a temporary file: the staged library's folders and files,
//! then [`ACCOUNT_FILE`]. Hidden entries — the library's lock file — are left out.

use std::fs::{self, File};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::Path;

use time::{OffsetDateTime, PrimitiveDateTime, UtcOffset};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

use super::{ACCOUNT_FILE, AccountFile};

/// What starts the name of an entry the export leaves out.
const HIDDEN_PREFIX: char = '.';

/// The zip of the library in `root` and of `account`, every entry dated `at`, in a temporary
/// file removed once closed, rewound; and its size. Blocks on the file system.
pub(super) fn write(
    root: &Path,
    account: &AccountFile,
    at: OffsetDateTime,
) -> io::Result<(File, u64)> {
    let mut zip = ZipWriter::new(tempfile::tempfile()?);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(entry_time(at));
    add_folder(&mut zip, (root, ""), options)?;
    zip.start_file(ACCOUNT_FILE, options)?;
    serde_json::to_writer_pretty(&mut zip, account)?;
    let mut file = zip.finish()?;
    let bytes = file.seek(SeekFrom::End(0))?;
    file.rewind()?;
    Ok((file, bytes))
}

/// The time of the entries of an export made `at`, in UTC.
fn entry_time(at: OffsetDateTime) -> DateTime {
    let utc = at.to_offset(UtcOffset::UTC);
    DateTime::try_from(PrimitiveDateTime::new(utc.date(), utc.time())).unwrap_or_default()
}

/// Adds the entries of `folder`, named after `prefix`, sorted by name, subfolders included.
fn add_folder<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    (folder, prefix): (&Path, &str),
    options: SimpleFileOptions,
) -> io::Result<()> {
    let mut entries = fs::read_dir(folder)?.collect::<io::Result<Vec<_>>>()?;
    entries.sort_by_key(fs::DirEntry::file_name);
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with(HIDDEN_PREFIX) {
            continue;
        }
        let path = format!("{prefix}{name}");
        if entry.file_type()?.is_dir() {
            zip.add_directory(format!("{path}/"), options)?;
            add_folder(zip, (&entry.path(), &format!("{path}/")), options)?;
        } else {
            zip.start_file(path, options)?;
            io::copy(&mut File::open(entry.path())?, zip)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use time::macros::datetime;
    use zip::ZipArchive;

    use super::*;

    /// A library folder, with a hidden file.
    fn library_folder() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("projects/p/animations")).unwrap();
        fs::write(root.path().join("library.json"), "{}").unwrap();
        fs::write(root.path().join("projects/p/project.json"), "{\"id\":1}").unwrap();
        fs::write(root.path().join(".lock"), "").unwrap();
        root
    }

    /// The `account.json` entry of `zip`: its date, and its content.
    fn account_entry(zip: &mut ZipArchive<File>) -> (Option<DateTime>, serde_json::Value) {
        let mut entry = zip.by_name(ACCOUNT_FILE).unwrap();
        let mut text = String::new();
        entry.read_to_string(&mut text).unwrap();
        (entry.last_modified(), serde_json::from_str(&text).unwrap())
    }

    #[test]
    fn the_zip_holds_the_folder_and_the_account_but_hidden_entries() {
        let root = library_folder();
        let account = AccountFile {
            email: "ada@example.com".to_owned(),
            language: "fr".to_owned(),
            plan: "free".to_owned(),
            created_at: datetime!(2026-09-01 12:00 UTC),
        };
        let at = datetime!(2026-09-26 10:30 UTC);

        let (file, bytes) = write(root.path(), &account, at).unwrap();

        assert_eq!(bytes, file.metadata().unwrap().len());
        let mut zip = ZipArchive::new(file).unwrap();
        let names: Vec<_> = zip.file_names().map(str::to_owned).collect();
        let expected = [
            "library.json",
            "projects/",
            "projects/p/",
            "projects/p/animations/",
            "projects/p/project.json",
            ACCOUNT_FILE,
        ];
        assert_eq!(names, expected);
        let (modified, json) = account_entry(&mut zip);
        assert_eq!(modified, Some(entry_time(at)));
        let expected = serde_json::json!({
            "email": "ada@example.com", "language": "fr", "plan": "free",
            "createdAt": "2026-09-01T12:00:00Z"
        });
        assert_eq!(json, expected);
    }
}
