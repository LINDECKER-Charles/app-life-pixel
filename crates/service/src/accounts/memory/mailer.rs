//! A mailer that keeps what it is given to send.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use async_trait::async_trait;
use tokio::sync::Notify;

use crate::accounts::ports::{Email, MailError, Mailer};

/// Keeps every email it is asked to send, in order, and wakes those waiting for one.
#[derive(Debug, Default)]
pub struct RecordingMailer {
    sent: Mutex<Vec<Email>>,
    arrived: Notify,
}

impl RecordingMailer {
    /// A mailer that has sent nothing yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The emails sent so far, in order.
    #[must_use]
    pub fn sent(&self) -> Vec<Email> {
        self.lock().clone()
    }

    /// The emails sent once there are at least `count`, or those sent within `timeout`: emails
    /// leave on tasks of their own.
    pub async fn wait_for(&self, count: usize, timeout: Duration) -> Vec<Email> {
        let waiting = async {
            loop {
                let arrived = self.arrived.notified();
                if self.lock().len() >= count {
                    return;
                }
                arrived.await;
            }
        };
        let _in_time = tokio::time::timeout(timeout, waiting).await;
        self.sent()
    }

    fn lock(&self) -> MutexGuard<'_, Vec<Email>> {
        self.sent.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[async_trait]
impl Mailer for RecordingMailer {
    async fn send(&self, email: Email) -> Result<(), MailError> {
        self.lock().push(email);
        self.arrived.notify_waiters();
        Ok(())
    }
}
