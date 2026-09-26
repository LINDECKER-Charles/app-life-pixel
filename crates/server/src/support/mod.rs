//! Support requests over HTTP (H9): the hosted stores the use cases keep requests and
//! screenshots in. The routes are in [`crate::routes::support`].

use std::sync::Arc;

use life_pixel_service::support::SupportStores;
use object_store::ObjectStore;
use sqlx::PgPool;

use crate::storage::{ObjectScreenshotStore, PostgresSupportStore};

/// The support stores of the hosted service: the requests in the database of `pool`, the
/// screenshots in `objects`.
#[must_use]
pub fn hosted_stores(pool: &PgPool, objects: Arc<dyn ObjectStore>) -> SupportStores {
    SupportStores {
        requests: Arc::new(PostgresSupportStore::new(pool.clone())),
        screenshots: Arc::new(ObjectScreenshotStore::new(objects)),
    }
}
