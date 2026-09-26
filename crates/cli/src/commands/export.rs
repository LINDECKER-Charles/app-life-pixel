//! `export`: an animation's files, written into a directory of the working tree —
//! `docs/v1/mcp-cli.md`'s "A4 — CLI". Unlike the `mcp` transport's `export` tool, this command
//! has no `--allow-dir`: the person running it already chose `--out` themselves.

use std::path::{Path, PathBuf};

use life_pixel_service::animation::ExportRequest;
use life_pixel_service::{AnimationId, CodedError, Owner};

use crate::args::ExportArgs;
use crate::errors::LocalWriteError;
use crate::local_write::write_files;
use crate::messages::Messages;
use crate::{AppError, library};

/// Runs `export`: writes the animation's files, then prints where each one landed.
///
/// # Errors
///
/// The library or the animation cannot be read, the export cannot be built, or a local write is
/// refused (`export.file_exists`, `export.symlink`, `export.write_failed`).
pub async fn run(args: ExportArgs, messages: &Messages) -> Result<(), AppError> {
    let path = library::resolve_path(args.library);
    let (_library, editing) = library::open(&path)?;
    let request = ExportRequest {
        id: AnimationId::from_uuid(args.id),
        format: args.format.into(),
        tag: args.tag,
        scale: args.scale,
    };
    let files = editing.export(&Owner::Local, request);
    let files = files.await.map_err(|error| CodedError::of(&error))?;
    let directory = resolve_directory(args.out.as_deref())?;
    let paths = write_files(&directory, &files, args.overwrite).map_err(CodedError::from)?;
    print_summary(&paths, messages);
    Ok(())
}

/// `--out`, relative to the working directory, created with its parents when missing; the
/// working directory itself when omitted.
fn resolve_directory(requested: Option<&Path>) -> Result<PathBuf, AppError> {
    let working_directory =
        std::env::current_dir().map_err(|error| AppError::Plain(error.to_string()))?;
    let target = requested.map_or_else(
        || working_directory.clone(),
        |relative| working_directory.join(relative),
    );
    std::fs::create_dir_all(&target)
        .map_err(|_| CodedError::from(LocalWriteError::DirectoryNotAllowed(target.clone())))?;
    let canonical = std::fs::canonicalize(&target)
        .map_err(|_| CodedError::from(LocalWriteError::DirectoryNotAllowed(target)))?;
    Ok(canonical)
}

fn print_summary(paths: &[PathBuf], messages: &Messages) {
    let count = paths.len().to_string();
    let directory = paths.first().and_then(|path| path.parent());
    let directory = directory
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    let params = [("count", count.as_str()), ("directory", directory.as_str())];
    println!("{}", messages.text("cli.export.summary", &params));
    for path in paths {
        println!("{}", path.display());
    }
}
