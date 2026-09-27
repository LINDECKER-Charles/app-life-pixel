//! Valid documents and records for the cases, made by `core`: an adapter that parses what it
//! stores reads the same title and size as the record says.

use bytes::Bytes;
use life_pixel_core::serialize::write_document;
use life_pixel_core::{Animation, Name, NewAnimation};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::ids::{AnimationId, ProjectId};
use crate::ports::library_store::{AnimationMeta, NewAnimationRecord, ProjectRecord};

/// The canvas side of the sample documents, in pixels.
const SAMPLE_SIDE: u16 = 4;
/// The name of the sample documents' layer.
const SAMPLE_LAYER: &str = "Layer";

/// A blank animation titled `title`, as `core` serializes it, and what lists show of it. A longer
/// title gives a longer document.
#[must_use]
pub fn sample_document(title: &str) -> (AnimationMeta, Bytes) {
    let animation = Animation::new(NewAnimation {
        title: Name::new(title).expect("a valid title"),
        width: SAMPLE_SIDE,
        height: SAMPLE_SIDE,
        layer_name: Name::new(SAMPLE_LAYER).expect("a valid layer name"),
        frame_duration_ms: None,
        palette: None,
    })
    .expect("a valid animation");
    let meta = AnimationMeta {
        title: animation.title().clone(),
        width: animation.width(),
        height: animation.height(),
        frame_count: u16::try_from(animation.frames().len()).expect("few frames"),
    };
    let document = write_document(&animation).expect("a small document");
    (meta, Bytes::from(document))
}

/// A new project named `name`, with a new id, created now.
#[must_use]
pub fn new_project(name: &str) -> ProjectRecord {
    let now = OffsetDateTime::now_utc();
    ProjectRecord {
        id: ProjectId::from_uuid(Uuid::now_v7()),
        name: Name::new(name).expect("a valid project name"),
        animation_count: 0,
        created_at: now,
        updated_at: now,
    }
}

/// A new animation titled `title` for `project`, with a new id, created now.
#[must_use]
pub fn new_animation(project: ProjectId, title: &str) -> NewAnimationRecord {
    let (meta, document) = sample_document(title);
    NewAnimationRecord {
        id: AnimationId::from_uuid(Uuid::now_v7()),
        project,
        meta,
        document,
        at: OffsetDateTime::now_utc(),
    }
}
