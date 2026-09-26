//! Reading the support queue: the requests with their age, one with its thread, its screenshot.

use bytes::Bytes;

use crate::admin::ports::{AdminRequest, AdminThread, QueueFilter};
use crate::admin::{Admin, AdminError};
use crate::paging::{Page, PageRequest};
use crate::support::SupportRequestId;

/// A request of the queue, with its age.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedRequest {
    /// The request.
    pub request: AdminRequest,
    /// How long ago it was sent, in whole seconds.
    pub age_seconds: i64,
}

/// A request's thread, with the request's age.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueuedThread {
    /// The thread.
    pub thread: AdminThread,
    /// How long ago the request was sent, in whole seconds.
    pub age_seconds: i64,
}

impl Admin {
    /// A page of the requests `filter` keeps, from the most recently updated, each with its age.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn support_queue(
        &self,
        filter: QueueFilter,
        page: PageRequest,
    ) -> Result<Page<QueuedRequest>, AdminError> {
        let page = self.support().queue(filter, page).await?;
        Ok(Page {
            items: page.items.into_iter().map(|item| self.aged(item)).collect(),
            next_cursor: page.next_cursor,
        })
    }

    /// The request `id` with its context and every message, internal notes included, and its
    /// age.
    ///
    /// # Errors
    ///
    /// `support.request_not_found`, `service.unavailable`.
    pub async fn support_thread(&self, id: SupportRequestId) -> Result<QueuedThread, AdminError> {
        let thread = self.support().thread(id).await?;
        let thread = thread.ok_or(AdminError::RequestNotFound)?;
        let age = self.now() - thread.request.request.created_at;
        Ok(QueuedThread {
            thread,
            age_seconds: age.whole_seconds().max(0),
        })
    }

    /// The screenshot of the request `id`, as the PNG it was stored as.
    ///
    /// # Errors
    ///
    /// `support.request_not_found`, `admin.screenshot_not_found`, `service.unavailable`.
    pub async fn support_screenshot(&self, id: SupportRequestId) -> Result<Bytes, AdminError> {
        let key = self.support().screenshot_key(id).await?;
        let key = key.ok_or(AdminError::RequestNotFound)?;
        let key = key.ok_or(AdminError::ScreenshotNotFound)?;
        let png = self.ports.screenshots.read(&key).await;
        png.map_err(|error| {
            tracing::error!(detail = %error.0, "a screenshot cannot be read");
            AdminError::Unavailable
        })
    }

    /// `request`, with its age now.
    pub(super) fn aged(&self, request: AdminRequest) -> QueuedRequest {
        let age = self.now() - request.request.created_at;
        QueuedRequest {
            request,
            age_seconds: age.whole_seconds().max(0),
        }
    }
}
