//! Support requests kept in memory.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use async_trait::async_trait;

use crate::ids::AccountId;
use crate::paging::{Cursor, Page, PageRequest, paginate};
use crate::support::ports::{
    NewSupportRequest, NewUserMessage, SupportMessage, SupportRequest, SupportStore,
    SupportStoreError, SupportThread, UserReply,
};
use crate::support::{SupportContext, SupportRequestId, SupportStatus};

/// A request as the store keeps it.
#[derive(Clone, Debug)]
struct Entry {
    account: AccountId,
    request: SupportRequest,
    context: SupportContext,
    screenshot_key: Option<String>,
    /// Each message, with whether it is an internal note.
    messages: Vec<(SupportMessage, bool)>,
}

/// Keeps requests in memory; tests change what only the team changes, and make it fail.
#[derive(Debug, Default)]
pub struct InMemorySupportStore {
    entries: Mutex<HashMap<SupportRequestId, Entry>>,
    is_unavailable: AtomicBool,
}

impl InMemorySupportStore {
    /// A store with no request yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes every call fail, or work again.
    pub fn set_unavailable(&self, is_unavailable: bool) {
        self.is_unavailable.store(is_unavailable, Ordering::SeqCst);
    }

    /// Sets the status of `id`, as the team does.
    pub fn set_status(&self, id: SupportRequestId, status: SupportStatus) {
        if let Some(entry) = self.lock().get_mut(&id) {
            entry.request.status = status;
        }
    }

    /// Adds `note` to `id` as an internal note of the team.
    pub fn add_internal_note(&self, id: SupportRequestId, note: SupportMessage) {
        if let Some(entry) = self.lock().get_mut(&id) {
            entry.messages.push((note, true));
        }
    }

    /// The context and screenshot key stored with `id`.
    #[must_use]
    pub fn stored(&self, id: SupportRequestId) -> Option<(SupportContext, Option<String>)> {
        let entries = self.lock();
        let entry = entries.get(&id)?;
        Some((entry.context.clone(), entry.screenshot_key.clone()))
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<SupportRequestId, Entry>> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn available(&self) -> Result<(), SupportStoreError> {
        if self.is_unavailable.load(Ordering::SeqCst) {
            return Err(SupportStoreError(
                "the in-memory store is set to fail".into(),
            ));
        }
        Ok(())
    }
}

#[async_trait]
impl SupportStore for InMemorySupportStore {
    async fn create(&self, new: NewSupportRequest) -> Result<(), SupportStoreError> {
        self.available()?;
        let entry = Entry {
            account: new.account,
            request: new.request.clone(),
            context: new.context,
            screenshot_key: new.screenshot_key,
            messages: vec![(new.message, false)],
        };
        self.lock().insert(new.request.id, entry);
        Ok(())
    }

    async fn list(
        &self,
        account: AccountId,
        page: PageRequest,
    ) -> Result<Page<SupportRequest>, SupportStoreError> {
        self.available()?;
        let entries = self.lock();
        let requests = entries
            .values()
            .filter(|entry| entry.account == account)
            .map(|entry| entry.request.clone())
            .collect();
        Ok(paginate(requests, &page, |request| Cursor {
            updated_at: request.updated_at,
            id: request.id.uuid(),
        }))
    }

    async fn thread(
        &self,
        account: AccountId,
        id: SupportRequestId,
    ) -> Result<Option<SupportThread>, SupportStoreError> {
        self.available()?;
        let entries = self.lock();
        let entry = entries.get(&id).filter(|entry| entry.account == account);
        Ok(entry.map(|entry| SupportThread {
            request: entry.request.clone(),
            messages: entry
                .messages
                .iter()
                .filter(|(_, is_internal)| !is_internal)
                .map(|(message, _)| message.clone())
                .collect(),
        }))
    }

    async fn add_user_message(
        &self,
        reply: NewUserMessage,
    ) -> Result<UserReply, SupportStoreError> {
        self.available()?;
        let mut entries = self.lock();
        let entry = entries.get_mut(&reply.request);
        let Some(entry) = entry.filter(|entry| entry.account == reply.account) else {
            return Ok(UserReply::NotFound);
        };
        let Some(status) = entry.request.status.after_user_reply() else {
            return Ok(UserReply::Refused);
        };
        entry.request.status = status;
        entry.request.updated_at = reply.message.created_at;
        entry.messages.push((reply.message, false));
        Ok(UserReply::Added { status })
    }
}
