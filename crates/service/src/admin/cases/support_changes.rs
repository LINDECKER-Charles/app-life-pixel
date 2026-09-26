//! What the team does to a request, each action audited: its status and assignee, its internal
//! notes, and the replies emailed to the person and shown in the app.

use std::sync::Arc;

use super::QueuedRequest;
use crate::accounts::ports::{Email, Message};
use crate::admin::ports::{AdminMessage, Recipient, RequestChange, TeamMessage};
use crate::admin::{Admin, AdminError, AdminIdentity, Assignee, AuditAction};
use crate::support::{Author, MessageBody, SupportRequestId, SupportStatus, THREAD_PATH};

/// A change of a request's status or assignee; at least one of them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestUpdate {
    /// The request.
    pub id: SupportRequestId,
    /// Its new status, if it changes.
    pub status: Option<SupportStatus>,
    /// Its new assignee — `Some(None)` for nobody —, if it changes.
    pub assigned_to: Option<Option<String>>,
}

/// A message of the team, as the admin wrote it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageDraft {
    /// The request.
    pub request: SupportRequestId,
    /// Its text.
    pub body: String,
    /// Whether it is a note the person never sees, rather than a reply.
    pub is_internal: bool,
}

impl Admin {
    /// Changes the status or the assignee of a request: the request after it, with its age.
    ///
    /// # Errors
    ///
    /// `request.malformed` when nothing changes or the assignee is not an admin id,
    /// `support.request_not_found`, `service.unavailable`.
    pub async fn update_request(
        &self,
        admin: &AdminIdentity,
        update: RequestUpdate,
    ) -> Result<QueuedRequest, AdminError> {
        if update.status.is_none() && update.assigned_to.is_none() {
            return Err(AdminError::Malformed);
        }
        let assigned_to = update.assigned_to.map(|assignee| {
            let parsed = assignee.as_deref().map(Assignee::parse);
            parsed.transpose()
        });
        let change = RequestChange {
            id: update.id,
            status: update.status,
            assigned_to: assigned_to.transpose()?,
            audit: self.draft(admin, (AuditAction::SupportUpdate, update.id.to_string())),
        };
        let request = self.support().update(change).await?;
        Ok(self.aged(request.ok_or(AdminError::RequestNotFound)?))
    }

    /// Adds a message of the team to a request. A reply moves the request to
    /// `waiting_for_user`, records the first response, and is emailed to the person with the link
    /// to the thread in the app; an internal note changes nothing else.
    ///
    /// # Errors
    ///
    /// `support.message_length`, `support.request_not_found`, `service.unavailable`.
    pub async fn post_message(
        &self,
        admin: &AdminIdentity,
        draft: MessageDraft,
    ) -> Result<AdminMessage, AdminError> {
        let body = MessageBody::parse(&draft.body)?;
        let action = if draft.is_internal {
            AuditAction::SupportNote
        } else {
            AuditAction::SupportReply
        };
        let audit = self.draft(admin, (action, draft.request.to_string()));
        let message = AdminMessage {
            id: self.ports.ids.new_id(),
            author: Author::Team,
            admin_id: Some(admin.id().to_owned()),
            body: body.into_string(),
            is_internal: draft.is_internal,
            created_at: audit.at,
        };
        let team_message = TeamMessage {
            request: draft.request,
            message: message.clone(),
            audit,
        };
        let recipient = self.support().add_team_message(team_message).await?;
        let recipient = recipient.ok_or(AdminError::RequestNotFound)?;
        if !draft.is_internal {
            self.email_reply(recipient, draft.request);
        }
        Ok(message)
    }

    /// Emails `recipient` that the request `id` was answered, on a task of its own; a failure is
    /// logged, never returned: the reply is in the app either way.
    fn email_reply(&self, recipient: Recipient, id: SupportRequestId) {
        let link = format!("{}{THREAD_PATH}{id}", self.settings.public_url);
        let email = Email {
            to: recipient.email,
            language: recipient.language,
            message: Message::SupportReply { link },
        };
        let mailer = Arc::clone(&self.ports.mailer);
        tokio::spawn(async move {
            if let Err(error) = mailer.send(email).await {
                tracing::warn!(%error, "a support reply was not emailed");
            }
        });
    }
}
