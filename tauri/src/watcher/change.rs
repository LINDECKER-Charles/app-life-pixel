//! Which projects and animations changed files concern, from the local library's layout:
//! `projects/<project-id>/project.json` and `projects/<project-id>/animations/<id>.json`.

use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::{Component, Path};

use serde::Serialize;
use uuid::Uuid;

/// The library's folder of projects, one folder per project.
const PROJECTS_FOLDER: &str = "projects";
/// A project's folder of animations, one document per animation.
const ANIMATIONS_FOLDER: &str = "animations";
/// The extension of an animation's document.
const DOCUMENT_EXTENSION: &str = "json";
/// What starts the name of the library's own files: its lock, and a write not yet renamed.
const HIDDEN_PREFIX: &str = ".";

/// The payload of `library-changed`: the projects and the animations whose files changed. Both
/// empty means the watcher lost track, and anything may have changed.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryChange {
    /// The projects concerned, those of the animations included.
    pub project_ids: BTreeSet<String>,
    /// The animations whose documents changed.
    pub animation_ids: BTreeSet<String>,
}

impl LibraryChange {
    /// Whether no project and no animation is named.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.project_ids.is_empty() && self.animation_ids.is_empty()
    }

    /// Records what `relative`, a path inside the library, concerns; a path outside its projects,
    /// or one of the library's own files, concerns nothing.
    pub fn record(&mut self, relative: &Path) {
        let mut names = relative.components().map(|component| match component {
            Component::Normal(name) => name,
            _ => OsStr::new(""),
        });
        if names.next() != Some(OsStr::new(PROJECTS_FOLDER)) {
            return;
        }
        let Some(project) = names.next().and_then(id_of) else {
            return;
        };
        let rest: Vec<&OsStr> = names.collect();
        if rest.iter().any(|name| is_hidden(name)) {
            return;
        }
        self.project_ids.insert(project);
        if let [folder, document] = rest.as_slice()
            && *folder == OsStr::new(ANIMATIONS_FOLDER)
        {
            self.animation_ids.extend(document_id(document));
        }
    }
}

/// The id a folder or file stem holds, when it is one.
fn id_of(name: &OsStr) -> Option<String> {
    let id = Uuid::try_parse(name.to_str()?).ok()?;
    Some(id.to_string())
}

/// The animation id of a document's file name, `<id>.json`.
fn document_id(name: &OsStr) -> Option<String> {
    let path = Path::new(name);
    if path.extension() != Some(OsStr::new(DOCUMENT_EXTENSION)) {
        return None;
    }
    id_of(path.file_stem()?)
}

/// Whether `name` is one of the library's own files, never a project's or an animation's.
fn is_hidden(name: &OsStr) -> bool {
    name.to_str()
        .is_some_and(|name| name.starts_with(HIDDEN_PREFIX))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROJECT: &str = "0190a4b2-7c3d-7e4f-8a5b-6c7d8e9fa0b1";
    const ANIMATION: &str = "0190a4b2-7c3d-7e4f-8a5b-6c7d8e9fa0b2";

    fn change_of(paths: &[&str]) -> LibraryChange {
        let mut change = LibraryChange::default();
        for path in paths {
            change.record(Path::new(path));
        }
        change
    }

    #[test]
    fn a_document_names_its_animation_and_its_project() {
        let change = change_of(&[&format!("projects/{PROJECT}/animations/{ANIMATION}.json")]);
        assert_eq!(change.project_ids, BTreeSet::from([PROJECT.to_owned()]));
        assert_eq!(change.animation_ids, BTreeSet::from([ANIMATION.to_owned()]));
    }

    #[test]
    fn a_project_file_or_folder_names_the_project_only() {
        let paths = [
            format!("projects/{PROJECT}"),
            format!("projects/{PROJECT}/project.json"),
            format!("projects/{PROJECT}/animations"),
        ];
        for path in paths {
            let change = change_of(&[&path]);
            assert_eq!(change.project_ids.len(), 1, "{path}");
            assert!(change.animation_ids.is_empty(), "{path}");
        }
    }

    #[test]
    fn the_library_s_own_and_unknown_files_concern_nothing() {
        let paths = [
            ".lock".to_owned(),
            "library.json".to_owned(),
            "projects".to_owned(),
            "projects/not-an-id/project.json".to_owned(),
            format!("projects/{PROJECT}/animations/.writing-a1b2c3"),
            format!("projects/{PROJECT}/.writing-d4e5f6"),
        ];
        for path in paths {
            assert!(change_of(&[&path]).is_empty(), "{path}");
        }
        let unknown = format!("projects/{PROJECT}/animations/notes.txt");
        assert!(change_of(&[&unknown]).animation_ids.is_empty());
    }
}
