//! A temporary local library, seeded directly through `service`, and the compiled `life-pixel`
//! binary run as a child process: the CLI's integration tests never touch the default library.

#![allow(clippy::unwrap_used, reason = "a panic is a failed test")]
#![allow(
    dead_code,
    reason = "shared by several test binaries, each using a subset"
)]

use std::path::Path;
use std::process::{Command, Output};

use life_pixel_service::local::LocalLibrary;
use life_pixel_service::ports::LibraryStore;
use life_pixel_service::testing::{new_animation, new_project};
use life_pixel_service::{AnimationId, Owner, ProjectId};
use tempfile::TempDir;

/// A library folder, kept alive for the test, with one project holding one animation.
pub struct TestLibrary {
    folder: TempDir,
    pub project: ProjectId,
    pub animation: AnimationId,
}

impl TestLibrary {
    /// A library with a project "Pets" holding a blank 4×4 animation titled `title`.
    pub async fn seeded(title: &str) -> Self {
        let folder = TempDir::new().unwrap();
        let store = LocalLibrary::create(folder.path()).unwrap();
        let project = new_project("Pets");
        store
            .create_project(&Owner::Local, project.clone())
            .await
            .unwrap();
        let new = new_animation(project.id, title);
        let created = store
            .create_animation(&Owner::Local, new, None)
            .await
            .unwrap();
        Self {
            folder,
            project: project.id,
            animation: created.id,
        }
    }

    /// An empty library: its folder is created, with no project and no animation.
    pub fn empty() -> Self {
        let folder = TempDir::new().unwrap();
        LocalLibrary::create(folder.path()).unwrap();
        Self {
            folder,
            project: ProjectId::from_uuid(uuid::Uuid::nil()),
            animation: AnimationId::from_uuid(uuid::Uuid::nil()),
        }
    }

    /// The library's folder.
    pub fn path(&self) -> &Path {
        self.folder.path()
    }
}

/// Runs the compiled `life-pixel` binary with `args`, from `directory`, `variables` set on the
/// child process only.
pub fn life_pixel(directory: &Path, args: &[&str], variables: &[(&str, &str)]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_life-pixel"))
        .args(args)
        .current_dir(directory)
        .envs(variables.iter().copied())
        .output()
        .unwrap()
}

/// The library argument, as a `Vec<&str>` ready to append to a command's arguments.
pub fn library_args(library: &TestLibrary) -> Vec<String> {
    vec!["--library".to_owned(), library.path().display().to_string()]
}

/// `output`'s stdout, as UTF-8 text.
pub fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// `output`'s stderr, as UTF-8 text.
pub fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// A directory the caller keeps alive, distinct from `library`'s own folder: `export`'s target.
pub fn scratch_dir() -> TempDir {
    TempDir::new().unwrap()
}
