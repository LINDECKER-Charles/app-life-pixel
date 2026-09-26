//! Reading one's requests, and replying to one.

use crate::ids::AccountId;
use crate::paging::{Page, PageRequest};
use crate::support::ports::{
    NewUserMessage, SupportMessage, SupportRequest, SupportThread, UserReply,
};
use crate::support::{MessageBody, Support, SupportError, SupportRequestId};

impl Support {
    /// The requests of `account`, from the most recently updated.
    ///
    /// # Errors
    ///
    /// `service.unavailable`.
    pub async fn list(
        &self,
        account: AccountId,
        page: PageRequest,
    ) -> Result<Page<SupportRequest>, SupportError> {
        Ok(self.requests().list(account, page).await?)
    }

    /// The request `id` of `account`, with its messages and without the team's internal notes.
    ///
    /// # Errors
    ///
    /// `support.request_not_found` when `account` has no such request, `service.unavailable`.
    pub async fn thread(
        &self,
        account: AccountId,
        id: SupportRequestId,
    ) -> Result<SupportThread, SupportError> {
        let thread = self.requests().thread(account, id).await?;
        thread.ok_or(SupportError::RequestNotFound)
    }

    /// Adds `body` to the request `id` of `account`, as the user: a request waiting for them goes
    /// back to the team.
    ///
    /// # Errors
    ///
    /// `support.message_length`, `support.request_not_found`, `support.request_closed`,
    /// `service.unavailable`.
    pub async fn reply(
        &self,
        account: AccountId,
        (id, body): (SupportRequestId, &str),
    ) -> Result<SupportMessage, SupportError> {
        let body = MessageBody::parse(body)?;
        let message = self.user_message(body, self.ports.clock.now());
        let reply = NewUserMessage {
            account,
            request: id,
            message: message.clone(),
        };
        match self.requests().add_user_message(reply).await? {
            UserReply::Added { .. } => Ok(message),
            UserReply::NotFound => Err(SupportError::RequestNotFound),
            UserReply::Refused => Err(SupportError::RequestClosed),
        }
    }
}
