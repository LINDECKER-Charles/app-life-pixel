//! An animation of the test account and the use cases over it.

use std::sync::Arc;

use bytes::Bytes;
use life_pixel_core::edit::{self, Operation};
use life_pixel_core::serialize::write_document;
use life_pixel_core::{Animation, NewAnimation, Rgba};
use life_pixel_service::animation::{
    AnimationEditing, AnimationView, DescribeRequest, EditingError, WriteFrameRequest,
};
use life_pixel_service::ports::LibraryStore;
use life_pixel_service::{AnimationId, Coded};
use serde_json::{Map, Value};

use crate::common::{ACCOUNT, FREE_STORAGE_BYTES, Harness, spec};

/// The use cases over an animation of [`ACCOUNT`].
pub struct Fixture {
    pub harness: Harness,
    pub editing: AnimationEditing,
    pub id: AnimationId,
}

impl Fixture {
    /// A blank `width × height` animation titled "Mascot", with the default palette.
    pub async fn blank(width: u16, height: u16) -> Self {
        Self::create(Harness::default(), spec("Mascot", width, height)).await
    }

    /// A blank animation of `spec` in a library whose free plan allows `quota` bytes.
    pub async fn with_quota(quota: u64, spec: NewAnimation) -> Self {
        Self::create(Harness::with_quota(quota), spec).await
    }

    /// A blank 4 × 4 animation titled "Mascot" in a library over `store`.
    pub async fn over(store: Arc<dyn LibraryStore>) -> Self {
        let spec = spec("Mascot", 4, 4);
        Self::create(Harness::over(store, FREE_STORAGE_BYTES), spec).await
    }

    /// The animation `animation`, imported.
    pub async fn imported(animation: &Animation) -> Self {
        let harness = Harness::default();
        let project = harness.project(&ACCOUNT, "Pets").await;
        let document = Bytes::from(write_document(animation).unwrap());
        let import = harness
            .library
            .import_animation(&ACCOUNT, project.id, document);
        let record = import.await.unwrap();
        Self::around(harness, record.id)
    }

    async fn create(harness: Harness, spec: NewAnimation) -> Self {
        let project = harness.project(&ACCOUNT, "Pets").await;
        let create = harness.library.create_animation(&ACCOUNT, project.id, spec);
        let record = create.await.unwrap();
        Self::around(harness, record.id)
    }

    fn around(harness: Harness, id: AnimationId) -> Self {
        let editing = AnimationEditing::new(harness.library.clone(), harness.events.clone());
        Self {
            harness,
            editing,
            id,
        }
    }

    /// The view, with the composites of the frames at `pixels`.
    pub async fn describe(&self, pixels: &[u16]) -> AnimationView {
        let request = DescribeRequest {
            id: self.id,
            pixels: Some(pixels.to_vec()),
        };
        self.editing.describe(&ACCOUNT, request).await.unwrap()
    }

    /// Writes `rows` on the top layer of the frame at `frame`.
    pub async fn write(&self, frame: u16, rows: &[&str]) -> Result<AnimationView, EditingError> {
        let request = WriteFrameRequest {
            id: self.id,
            frame,
            layer: None,
            grid: grid(rows),
        };
        self.editing.write_frame(&ACCOUNT, request).await
    }

    /// The version of the animation's document now.
    pub async fn version(&self) -> u64 {
        let record = self.harness.library.get_animation(&ACCOUNT, self.id);
        record.await.unwrap().version
    }
}

/// A 4 × 4 animation with a second layer, "Top", above "Layer 1".
pub fn two_layers() -> Animation {
    let mut animation = Animation::new(spec("Mascot", 4, 4)).unwrap();
    let add = Operation::AddLayer {
        position: 1,
        name: "Top".to_owned(),
    };
    edit::apply(&mut animation, &add).unwrap();
    animation
}

/// The colours of `#rrggbbaa` texts.
pub fn colours(texts: &[&str]) -> Vec<Rgba> {
    texts.iter().map(|text| text.parse().unwrap()).collect()
}

/// Owned rows of a text grid.
pub fn grid(rows: &[&str]) -> Vec<String> {
    rows.iter().map(|&row| row.to_owned()).collect()
}

/// Asserts that `error` has the code `code` and the parameters `params`.
pub fn assert_coded(error: &EditingError, code: &str, params: Value) {
    assert_eq!(error.code(), code, "{error:?}");
    let expected: Map<String, Value> = match params {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    assert_eq!(error.params(), expected, "{error:?}");
}
