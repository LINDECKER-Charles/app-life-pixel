//! `TestMailbox`: a recipient of its own, and Mailpit's API searched for the messages sent to it.

use std::time::Duration;

use serde::Deserialize;

use super::env::{TestSetupError, variable};

/// The domain of the test recipients: `.invalid` never delivers.
pub const TEST_MAIL_DOMAIN: &str = "test.life-pixel.invalid";
/// The variable naming Mailpit's web and API address.
const MAILPIT_URL: &str = "LP_TEST_MAILPIT_URL";
/// How often `wait_for_message` asks Mailpit again.
const POLL_PERIOD: Duration = Duration::from_millis(100);

/// A message Mailpit received.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MailMessage {
    /// Mailpit's id of the message.
    #[serde(rename = "ID")]
    pub id: String,
    /// The subject.
    pub subject: String,
    /// The plain-text body.
    #[serde(default)]
    pub text: String,
    /// The HTML body.
    #[serde(default, rename = "HTML")]
    pub html: String,
}

/// The messages of a search.
#[derive(Deserialize)]
struct Search {
    messages: Vec<Summary>,
}

/// A message, as a search lists it.
#[derive(Deserialize)]
struct Summary {
    #[serde(rename = "ID")]
    id: String,
}

/// A recipient of its own on the local stack's Mailpit.
pub struct TestMailbox {
    address: String,
    api: String,
    client: reqwest::Client,
}

impl TestMailbox {
    /// A new recipient, `lp-test-<16 random hexadecimal digits>@test.life-pixel.invalid`.
    ///
    /// # Errors
    ///
    /// When `LP_TEST_MAILPIT_URL` is missing.
    pub fn new() -> Result<Self, TestSetupError> {
        let api = variable(MAILPIT_URL)?.trim_end_matches('/').to_owned();
        let address = format!("lp-test-{:016x}@{TEST_MAIL_DOMAIN}", rand::random::<u64>());
        Ok(Self {
            address,
            api,
            client: reqwest::Client::new(),
        })
    }

    /// The recipient's address.
    #[must_use]
    pub fn address(&self) -> &str {
        &self.address
    }

    /// The messages sent to the recipient so far, the most recent first.
    ///
    /// # Errors
    ///
    /// When Mailpit's API fails.
    pub async fn messages(&self) -> Result<Vec<MailMessage>, TestSetupError> {
        let search: Search = self
            .client
            .get(format!("{}/api/v1/search", self.api))
            .query(&[("query", format!("to:\"{}\"", self.address))])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let mut messages = Vec::with_capacity(search.messages.len());
        for summary in search.messages {
            messages.push(self.message(&summary.id).await?);
        }
        Ok(messages)
    }

    /// The most recent message sent to the recipient, waiting up to `timeout` for one.
    ///
    /// # Errors
    ///
    /// When none arrives in time, or Mailpit's API fails.
    pub async fn wait_for_message(&self, timeout: Duration) -> Result<MailMessage, TestSetupError> {
        let waiting = async {
            loop {
                if let Some(message) = self.messages().await?.into_iter().next() {
                    return Ok(message);
                }
                tokio::time::sleep(POLL_PERIOD).await;
            }
        };
        let waited = tokio::time::timeout(timeout, waiting).await;
        waited.unwrap_or_else(|_| Err(TestSetupError::NoMessage(self.address.clone())))
    }

    async fn message(&self, id: &str) -> Result<MailMessage, TestSetupError> {
        let url = format!("{}/api/v1/message/{id}", self.api);
        let response = self.client.get(url).send().await?.error_for_status()?;
        Ok(response.json().await?)
    }
}
