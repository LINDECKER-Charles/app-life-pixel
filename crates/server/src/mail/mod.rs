//! Email: the `Mailer` of `service::accounts` over SMTP, and the texts it renders from the
//! catalogues' `email.` keys — plain text, and a minimal HTML part with the link.

mod html;
mod smtp;
mod templates;

pub use smtp::{MailSetupError, SmtpMailer};
pub use templates::{EmailTemplates, FALLBACK_LANGUAGE, RenderedEmail, TemplateError};
