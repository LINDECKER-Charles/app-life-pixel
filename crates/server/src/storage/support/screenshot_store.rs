//! `ObjectScreenshotStore`: support screenshots in object storage, at
//! `support/<request>/screenshot.png`.

use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use life_pixel_service::support::SupportRequestId;
use life_pixel_service::support::ports::{ScreenshotStore, SupportStoreError};
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt, PutPayload};

use crate::storage::keys::screenshot_key;

/// The screenshots of an object storage.
#[derive(Clone)]
pub struct ObjectScreenshotStore {
    objects: Arc<dyn ObjectStore>,
}

impl ObjectScreenshotStore {
    /// The store over `objects`.
    #[must_use]
    pub fn new(objects: Arc<dyn ObjectStore>) -> Self {
        Self { objects }
    }
}

/// The failure of the object storage.
fn objects(error: object_store::Error) -> SupportStoreError {
    SupportStoreError(format!("object storage: {error}"))
}

#[async_trait]
impl ScreenshotStore for ObjectScreenshotStore {
    async fn put(
        &self,
        request: SupportRequestId,
        png: Bytes,
    ) -> Result<String, SupportStoreError> {
        let key = screenshot_key(request);
        let payload = PutPayload::from_bytes(png);
        self.objects.put(&key, payload).await.map_err(objects)?;
        Ok(key.to_string())
    }

    async fn read(&self, key: &str) -> Result<Bytes, SupportStoreError> {
        let object = self.objects.get(&Path::from(key)).await.map_err(objects)?;
        object.bytes().await.map_err(objects)
    }

    async fn delete(&self, key: &str) -> Result<(), SupportStoreError> {
        self.objects.delete(&Path::from(key)).await.map_err(objects)
    }
}
