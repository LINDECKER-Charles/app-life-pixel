//! Object storage whose deletions fail on demand: a write whose cleanup fails leaves an orphan
//! for the sweeper.

use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use futures_util::StreamExt;
use futures_util::stream::BoxStream;
use object_store::path::Path;
use object_store::{
    CopyOptions, GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, ObjectStore,
    PutMultipartOptions, PutOptions, PutPayload, PutResult, Result,
};

/// The objects of another store, whose deletions fail while [`FailingDeletes::fail`] says so.
#[derive(Debug)]
pub struct FailingDeletes {
    inner: Arc<dyn ObjectStore>,
    is_failing: AtomicBool,
}

impl FailingDeletes {
    /// `inner`, whose deletions succeed until told otherwise.
    pub fn new(inner: Arc<dyn ObjectStore>) -> Self {
        Self {
            inner,
            is_failing: AtomicBool::new(false),
        }
    }

    /// Makes the deletions fail, or succeed again.
    pub fn fail(&self, is_failing: bool) {
        self.is_failing.store(is_failing, Ordering::SeqCst);
    }
}

impl fmt::Display for FailingDeletes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "FailingDeletes({})", self.inner)
    }
}

#[async_trait]
impl ObjectStore for FailingDeletes {
    async fn put_opts(
        &self,
        location: &Path,
        payload: PutPayload,
        opts: PutOptions,
    ) -> Result<PutResult> {
        self.inner.put_opts(location, payload, opts).await
    }

    async fn put_multipart_opts(
        &self,
        location: &Path,
        opts: PutMultipartOptions,
    ) -> Result<Box<dyn MultipartUpload>> {
        self.inner.put_multipart_opts(location, opts).await
    }

    async fn get_opts(&self, location: &Path, options: GetOptions) -> Result<GetResult> {
        self.inner.get_opts(location, options).await
    }

    fn delete_stream(
        &self,
        locations: BoxStream<'static, Result<Path>>,
    ) -> BoxStream<'static, Result<Path>> {
        if !self.is_failing.load(Ordering::SeqCst) {
            return self.inner.delete_stream(locations);
        }
        locations
            .map(|location| {
                Err(object_store::Error::Generic {
                    store: "FailingDeletes",
                    source: format!("injected failure deleting {}", location?).into(),
                })
            })
            .boxed()
    }

    fn list(&self, prefix: Option<&Path>) -> BoxStream<'static, Result<ObjectMeta>> {
        self.inner.list(prefix)
    }

    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> Result<ListResult> {
        self.inner.list_with_delimiter(prefix).await
    }

    async fn copy_opts(&self, from: &Path, to: &Path, options: CopyOptions) -> Result<()> {
        self.inner.copy_opts(from, to, options).await
    }
}
