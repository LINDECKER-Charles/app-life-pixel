//! Sending a request: its values checked, its screenshot re-encoded and stored first, then the
//! request and its first message together.

use bytes::Bytes;
use tokio::task::JoinError;

use crate::ids::AccountId;
use crate::ports::ProductEvent;
use crate::support::ports::{NewSupportRequest, SupportRequest, SupportThread};
use crate::support::{
    Category, MessageBody, Screenshot, ScreenshotRejection, Support, SupportContext, SupportError,
    SupportRequestId, SupportStatus,
};

/// A request was sent; property [`CATEGORY`].
const SUPPORT_REQUEST_CREATED: &str = "support_request_created";
/// The category of the request.
const CATEGORY: &str = "category";

/// What a person sends, as the form's parts came.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SupportSubmission {
    /// The category's value, `bug` for example.
    pub category: String,
    /// The message.
    pub message: String,
    /// The context, as JSON.
    pub context: Vec<u8>,
    /// The screenshot, as uploaded, if any.
    pub screenshot: Option<Bytes>,
}

/// The values of a submission, checked.
struct Checked {
    category: Category,
    body: MessageBody,
    context: SupportContext,
    screenshot: Option<Screenshot>,
}

impl Support {
    /// Sends the request of `submission` for `account`: its first message is the message, and
    /// its screenshot is re-encoded before it is stored. Records `support_request_created`.
    ///
    /// # Errors
    ///
    /// `support.category`, `support.message_length`, `request.malformed` for the context,
    /// `support.screenshot`, `service.unavailable`.
    pub async fn create(
        &self,
        account: AccountId,
        submission: SupportSubmission,
    ) -> Result<SupportThread, SupportError> {
        let mut checked = self.check(submission).await?;
        let id = SupportRequestId::from_uuid(self.new_id());
        let screenshot_key = self.store_screenshot(id, checked.screenshot.take()).await?;
        let new = self.new_request((account, id), (checked, screenshot_key));
        if let Err(error) = self.requests().create(new.clone()).await {
            self.discard_screenshot(new.screenshot_key).await;
            return Err(error.into());
        }
        self.record_created(account, new.request.category);
        Ok(SupportThread {
            request: new.request,
            messages: vec![new.message],
        })
    }

    /// The values of `submission`, in the order the form shows them, and its screenshot
    /// prepared.
    async fn check(&self, submission: SupportSubmission) -> Result<Checked, SupportError> {
        let category = Category::parse(&submission.category)?;
        let body = MessageBody::parse(&submission.message)?;
        let context = SupportContext::parse(&submission.context)?;
        let screenshot = match submission.screenshot {
            Some(upload) => Some(self.prepare(upload).await?),
            None => None,
        };
        Ok(Checked {
            category,
            body,
            context,
            screenshot,
        })
    }

    /// The request `id` of `account`, new, sent now with the values of `checked`.
    fn new_request(
        &self,
        (account, id): (AccountId, SupportRequestId),
        (checked, screenshot_key): (Checked, Option<String>),
    ) -> NewSupportRequest {
        let now = self.ports.clock.now();
        let request = SupportRequest {
            id,
            category: checked.category,
            status: SupportStatus::New,
            has_screenshot: screenshot_key.is_some(),
            created_at: now,
            updated_at: now,
        };
        NewSupportRequest {
            account,
            request,
            context: checked.context,
            screenshot_key,
            message: self.user_message(checked.body, now),
        }
    }

    /// The screenshot of `upload`, prepared on a blocking thread once a decode is free.
    async fn prepare(&self, upload: Bytes) -> Result<Screenshot, SupportError> {
        let _permit = self
            .decodes
            .acquire()
            .await
            .map_err(|_| SupportError::Unavailable)?;
        let prepared = tokio::task::spawn_blocking(move || Screenshot::prepare(&upload)).await;
        prepared_or_refused(prepared)
    }

    /// Stores `screenshot` for the request `id`: its key, or `None` without one.
    async fn store_screenshot(
        &self,
        id: SupportRequestId,
        screenshot: Option<Screenshot>,
    ) -> Result<Option<String>, SupportError> {
        let Some(screenshot) = screenshot else {
            return Ok(None);
        };
        Ok(Some(
            self.screenshots().put(id, screenshot.into_png()).await?,
        ))
    }

    /// Deletes the screenshot at `key` of a request that was not stored; a failure is logged,
    /// and the server's sweeper collects it later.
    async fn discard_screenshot(&self, key: Option<String>) {
        let Some(key) = key else { return };
        if let Err(error) = self.screenshots().delete(&key).await {
            tracing::warn!(detail = %error.0, "a support screenshot could not be deleted");
        }
    }

    fn record_created(&self, account: AccountId, category: Category) {
        self.ports.events.record(ProductEvent {
            name: SUPPORT_REQUEST_CREATED,
            account: Some(account),
            properties: vec![(CATEGORY, category.as_str().to_owned())],
        });
    }
}

/// The screenshot a blocking preparation gave, or why there is none.
fn prepared_or_refused(
    prepared: Result<Result<Screenshot, ScreenshotRejection>, JoinError>,
) -> Result<Screenshot, SupportError> {
    prepared.map_err(failed)?.map_err(refused)
}

/// `support.screenshot`, the refusal logged without the upload.
fn refused(rejection: ScreenshotRejection) -> SupportError {
    tracing::info!(%rejection, "a support screenshot was refused");
    SupportError::Screenshot
}

/// `service.unavailable`: the preparation's thread failed.
fn failed(error: JoinError) -> SupportError {
    tracing::error!(%error, "a support screenshot could not be prepared");
    SupportError::Unavailable
}
