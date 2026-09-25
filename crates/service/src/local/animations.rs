//! Reading animations: a record from each document file — its dates the file's, its title and
//! size from parsing it —, left out with a warning when the file cannot be read or parsed.

use std::fs::{File, Metadata};
use std::io::{self, Read as _};
use std::path::Path;

use bytes::Bytes;
use time::OffsetDateTime;

use super::documents::{Summary, summarize, version_of};
use super::files::is_not_found;
use super::folder::{Folder, unavailable};
use crate::ids::{AnimationId, ProjectId};
use crate::paging::{Cursor, Page, PageRequest, paginate};
use crate::ports::library_store::{AnimationFilter, AnimationRecord, StoreError};

/// A document file, opened, and where it belongs.
struct Opened<'a> {
    path: &'a Path,
    project: ProjectId,
    id: AnimationId,
    file: File,
    metadata: Metadata,
}

impl Folder {
    /// The animation `id`.
    pub(super) fn animation(&self, id: AnimationId) -> Result<AnimationRecord, StoreError> {
        let project = self.project_of(id)?;
        self.record(project, id)?
            .ok_or(StoreError::AnimationNotFound)
    }

    /// The animation `id` and its document.
    pub(super) fn read_document(
        &self,
        id: AnimationId,
    ) -> Result<(AnimationRecord, Bytes), StoreError> {
        let project = self.project_of(id)?;
        let path = self.document_file(project, id);
        let Some(opened) = self.open(&path, (project, id))? else {
            return Err(StoreError::AnimationNotFound);
        };
        let bytes = read_all(&opened.file).map_err(unavailable)?;
        let summary = self.summary(&opened, || Ok(bytes.clone()));
        let summary = summary.ok_or(StoreError::AnimationNotFound)?;
        // The version of the bytes read, whatever the cache holds.
        let summary = Summary {
            version: version_of(&bytes),
            ..summary
        };
        Ok((record(&opened, summary).map_err(unavailable)?, bytes.into()))
    }

    /// A page of the animations `filter` keeps.
    pub(super) fn list_animations(
        &self,
        filter: &AnimationFilter,
        page: &PageRequest,
    ) -> Result<Page<AnimationRecord>, StoreError> {
        let projects = match filter.project {
            Some(project) => vec![project],
            None => self.project_ids()?,
        };
        let query = filter.query.as_ref().map(|query| query.to_lowercase());
        let mut records = Vec::new();
        for project in projects {
            records.extend(self.records_of(project)?);
        }
        records.retain(|record| {
            let title = record.meta.title.as_str().to_lowercase();
            query.as_ref().is_none_or(|query| title.contains(query))
        });
        Ok(paginate(records, page, animation_cursor))
    }

    /// The bytes of every document.
    pub(super) fn usage(&self) -> Result<u64, StoreError> {
        let mut used: u64 = 0;
        for project in self.project_ids()? {
            let records = self.records_of(project)?;
            let sizes = records.iter().map(|record| record.document_bytes);
            used = used.saturating_add(sizes.sum());
        }
        Ok(used)
    }

    /// The animations of `project`, none when it is not a project.
    pub(super) fn records_of(
        &self,
        project: ProjectId,
    ) -> Result<Vec<AnimationRecord>, StoreError> {
        if !self.project_file(project).is_file() {
            return Ok(Vec::new());
        }
        let mut records = Vec::new();
        for id in self.animation_ids(project)? {
            records.extend(self.record(project, id)?);
        }
        Ok(records)
    }

    /// The animation `id` of `project`; `None` when its file is gone, unreadable or malformed.
    pub(super) fn record(
        &self,
        project: ProjectId,
        id: AnimationId,
    ) -> Result<Option<AnimationRecord>, StoreError> {
        let path = self.document_file(project, id);
        let Some(opened) = self.open(&path, (project, id))? else {
            return Ok(None);
        };
        let Some(summary) = self.summary(&opened, || read_all(&opened.file)) else {
            return Ok(None);
        };
        record(&opened, summary).map(Some).map_err(unavailable)
    }

    /// The document file at `path`, `None` when it is gone or cannot be opened.
    fn open<'a>(
        &self,
        path: &'a Path,
        (project, id): (ProjectId, AnimationId),
    ) -> Result<Option<Opened<'a>>, StoreError> {
        let opened = File::open(path).and_then(|file| Ok((file.metadata()?, file)));
        match opened {
            Ok((metadata, file)) => Ok(Some(Opened {
                path,
                project,
                id,
                file,
                metadata,
            })),
            Err(error) if is_not_found(&error) => Ok(None),
            Err(error) => {
                tracing::warn!(path = %path.display(), %error, "animation left out: unreadable");
                Ok(None)
            }
        }
    }

    /// What the opened file says: cached, or parsed from the bytes `read` gives and kept.
    fn summary(
        &self,
        opened: &Opened<'_>,
        read: impl FnOnce() -> io::Result<Vec<u8>>,
    ) -> Option<Summary> {
        if let Some(summary) = self.cache.get(opened.path, &opened.metadata) {
            return Some(summary);
        }
        let path = opened.path.display();
        let bytes = read()
            .inspect_err(|error| tracing::warn!(%path, %error, "animation left out: unreadable"))
            .ok()?;
        let Some(summary) = summarize(&bytes) else {
            tracing::warn!(%path, "animation left out: the document does not parse");
            return None;
        };
        self.cache
            .insert(opened.path, (&opened.metadata, summary.clone()));
        Some(summary)
    }
}

/// The record of the opened file.
fn record(opened: &Opened<'_>, summary: Summary) -> io::Result<AnimationRecord> {
    let modified = opened.metadata.modified()?;
    let created = opened.metadata.created().unwrap_or(modified);
    Ok(AnimationRecord {
        id: opened.id,
        project: opened.project,
        meta: summary.meta,
        document_bytes: opened.metadata.len(),
        version: summary.version,
        created_at: OffsetDateTime::from(created),
        updated_at: OffsetDateTime::from(modified),
    })
}

/// The bytes of `file`.
fn read_all(mut file: &File) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Where an animation stands in a list.
fn animation_cursor(animation: &AnimationRecord) -> Cursor {
    Cursor {
        updated_at: animation.updated_at,
        id: animation.id.uuid(),
    }
}
