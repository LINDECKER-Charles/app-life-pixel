//! `SmtpMailer`: the emails of the accounts, rendered from the catalogues and sent over the SMTP
//! server of `LP_SMTP_URL`.

use std::sync::Arc;

use async_trait::async_trait;
use lettre::message::{Mailbox, MultiPart};
use lettre::transport::smtp::Error as SmtpError;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message as SmtpMessage, Tokio1Executor};
use life_pixel_service::accounts::ports::{Email, MailError, Mailer};
use thiserror::Error;

use super::templates::EmailTemplates;
use crate::config::MailConfig;

/// Why the mailer could not be set up at start.
#[derive(Debug, Error)]
pub enum MailSetupError {
    /// `LP_SMTP_URL` is not a URL the SMTP client accepts; the URL is not repeated, since it may
    /// hold credentials.
    #[error("LP_SMTP_URL is not a valid SMTP URL")]
    Url,
    /// `LP_MAIL_FROM` is not an address, with or without a name.
    #[error("LP_MAIL_FROM is not an address such as Life Pixel <no-reply@lifepixel.tech>")]
    From,
}

/// Sends the accounts' emails over SMTP, through a pool of connections.
#[derive(Clone)]
pub struct SmtpMailer {
    transport: AsyncSmtpTransport<Tokio1Executor>,
    from: Mailbox,
    templates: Arc<EmailTemplates>,
}

impl SmtpMailer {
    /// The mailer of `config`, rendering from `templates`. It connects on the first email.
    ///
    /// # Errors
    ///
    /// When the URL or the sender does not parse.
    pub fn new(config: &MailConfig, templates: EmailTemplates) -> Result<Self, MailSetupError> {
        let transport = AsyncSmtpTransport::<Tokio1Executor>::from_url(config.smtp_url.expose())
            .map_err(|_| MailSetupError::Url)?
            .build();
        let from = config.from.parse().map_err(|_| MailSetupError::From)?;
        Ok(Self {
            transport,
            from,
            templates: Arc::new(templates),
        })
    }

    /// The message of `email`, rendered.
    fn compose(&self, email: &Email) -> Result<SmtpMessage, MailError> {
        let rendered = self.templates.render(email)?;
        let to: Mailbox = email
            .to
            .parse()
            .map_err(|_| MailError("the recipient is not an address".to_owned()))?;
        SmtpMessage::builder()
            .from(self.from.clone())
            .to(to)
            .subject(rendered.subject)
            .multipart(MultiPart::alternative_plain_html(
                rendered.text,
                rendered.html,
            ))
            .map_err(|error| MailError(format!("the message cannot be built: {error}")))
    }
}

#[async_trait]
impl Mailer for SmtpMailer {
    async fn send(&self, email: Email) -> Result<(), MailError> {
        let message = self.compose(&email)?;
        self.transport
            .send(message)
            .await
            .map_err(|error| smtp_failure(&error))?;
        Ok(())
    }
}

/// What went wrong with the SMTP server, said without the server's own words: they may repeat
/// the recipient's address.
fn smtp_failure(error: &SmtpError) -> MailError {
    let kind = if error.is_permanent() {
        "a permanent refusal"
    } else if error.is_transient() {
        "a transient refusal"
    } else if error.is_timeout() {
        "a timeout"
    } else if error.is_tls() {
        "a TLS failure"
    } else {
        "a connection failure"
    };
    let status = error
        .status()
        .map(|code| format!(" ({code})"))
        .unwrap_or_default();
    MailError(format!("the SMTP server: {kind}{status}"))
}
