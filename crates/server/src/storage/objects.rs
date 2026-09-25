//! Object storage from `LP_STORAGE_URL`: an S3-compatible bucket with the `LP_S3_*` settings, or
//! a local folder for self-hosting.

use std::io;
use std::path::Path;
use std::sync::Arc;

use object_store::ObjectStore;
use object_store::aws::AmazonS3Builder;
use object_store::local::LocalFileSystem;
use thiserror::Error;

use crate::config::{S3Config, StorageConfig};

/// The scheme of an endpoint without TLS, such as S3Mock's.
const PLAIN_HTTP: &str = "http://";

/// Why the object storage could not be set up.
#[derive(Debug, Error)]
pub enum ObjectStoreSetupError {
    /// The folder of a `file://` URL cannot be created.
    #[error("LP_STORAGE_URL: the folder cannot be created: {0}")]
    Folder(#[from] io::Error),
    /// The store rejected its settings.
    #[error("LP_STORAGE_URL: {0}")]
    Store(#[from] object_store::Error),
}

/// The object storage `config` names.
///
/// # Errors
///
/// When the bucket's settings are rejected, or the folder cannot be created.
pub fn object_store(config: &StorageConfig) -> Result<Arc<dyn ObjectStore>, ObjectStoreSetupError> {
    match config {
        StorageConfig::S3 { bucket, settings } => Ok(Arc::new(s3(bucket, settings)?)),
        StorageConfig::File { root } => Ok(Arc::new(folder(root)?)),
    }
}

fn s3(bucket: &str, settings: &S3Config) -> Result<impl ObjectStore, object_store::Error> {
    AmazonS3Builder::new()
        .with_bucket_name(bucket)
        .with_endpoint(&settings.endpoint)
        .with_region(&settings.region)
        .with_access_key_id(settings.access_key_id.expose())
        .with_secret_access_key(settings.secret_access_key.expose())
        .with_virtual_hosted_style_request(!settings.path_style)
        .with_allow_http(settings.endpoint.starts_with(PLAIN_HTTP))
        .build()
}

/// The folder `root`, created if missing; the folders a deletion empties are removed.
fn folder(root: &Path) -> Result<impl ObjectStore, ObjectStoreSetupError> {
    std::fs::create_dir_all(root)?;
    Ok(LocalFileSystem::new_with_prefix(root)?.with_automatic_cleanup(true))
}
