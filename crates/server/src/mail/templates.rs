//! The emails' texts: the `email.` keys of every catalogue of `LP_I18N_DIR`, rendered in the
//! account's language, English when a key is missing there, with their `{link}` replaced.

use std::collections::HashMap;
use std::io;
use std::path::Path;

use life_pixel_service::accounts::ports::{Email, MailError};
use serde::Deserialize;
use serde_json::{Map, Value};
use thiserror::Error;

use super::html;

/// The catalogue that lists the languages.
const LANGUAGES_FILE: &str = "languages.json";
/// The prefix of the emails' keys.
const EMAIL_PREFIX: &str = "email.";
/// The language of a key missing from the account's.
pub const FALLBACK_LANGUAGE: &str = "en";
/// The argument an email's link replaces.
const LINK_ARGUMENT: &str = "{link}";

/// Why the templates could not be read at start.
#[derive(Debug, Error)]
pub enum TemplateError {
    /// A file of the folder could not be read.
    #[error("cannot read {file}: {source}")]
    Read {
        /// The file's name.
        file: String,
        /// The failure.
        source: io::Error,
    },
    /// A file is not what it should be: a list of languages, or a flat catalogue.
    #[error("{file} does not parse")]
    Parse {
        /// The file's name.
        file: String,
    },
}

/// An email ready to leave: its subject, its text and its HTML part.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderedEmail {
    /// The subject.
    pub subject: String,
    /// The plain text.
    pub text: String,
    /// The minimal HTML part, the link clickable.
    pub html: String,
}

#[derive(Deserialize)]
struct Language {
    code: String,
}

/// The `email.` messages of every language, by language code and key.
#[derive(Clone, Debug, Default)]
pub struct EmailTemplates {
    messages: HashMap<String, HashMap<String, String>>,
}

impl EmailTemplates {
    /// Reads `languages.json` from `dir`, then the `email.` keys of each catalogue it lists.
    ///
    /// # Errors
    ///
    /// When a file cannot be read or does not parse.
    pub fn load(dir: &Path) -> Result<Self, TemplateError> {
        let languages: Vec<Language> = read_json(dir, LANGUAGES_FILE)?;
        let mut templates = Self::default();
        for Language { code } in languages {
            let catalogue: Map<String, Value> = read_json(dir, &format!("{code}.json"))?;
            templates.add(code, catalogue);
        }
        Ok(templates)
    }

    /// Adds the `email.` messages of `catalogue`, the catalogue of `language`.
    pub fn add(&mut self, language: String, catalogue: Map<String, Value>) {
        let emails = catalogue.into_iter().filter_map(|(key, message)| {
            let message = message.as_str()?.to_owned();
            key.starts_with(EMAIL_PREFIX).then_some((key, message))
        });
        self.messages.entry(language).or_default().extend(emails);
    }

    /// The subject, text and HTML of `email`, in its language or else in English.
    ///
    /// # Errors
    ///
    /// When neither catalogue has the message's subject and body.
    pub fn render(&self, email: &Email) -> Result<RenderedEmail, MailError> {
        let key = email.message.key();
        let (language, subject) = self.find(&email.language, &format!("email.{key}.subject"))?;
        let (_, body) = self.find(language, &format!("email.{key}.body"))?;
        let link = email.message.link().unwrap_or_default();
        Ok(RenderedEmail {
            subject: subject.replace(LINK_ARGUMENT, link),
            text: body.replace(LINK_ARGUMENT, link),
            html: html::render(language, body, link),
        })
    }

    /// The language that has `key` — `language`, else English — and its message.
    fn find<'a>(&'a self, language: &'a str, key: &str) -> Result<(&'a str, &'a str), MailError> {
        [language, FALLBACK_LANGUAGE]
            .into_iter()
            .find_map(|code| {
                let message = self.messages.get(code)?.get(key)?;
                Some((code, message.as_str()))
            })
            .ok_or_else(|| MailError(format!("no message {key} in any catalogue")))
    }
}

/// The JSON file `name` of `dir`.
fn read_json<T: serde::de::DeserializeOwned>(dir: &Path, name: &str) -> Result<T, TemplateError> {
    let bytes = std::fs::read(dir.join(name)).map_err(|source| TemplateError::Read {
        file: name.to_owned(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|_| TemplateError::Parse {
        file: name.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use life_pixel_service::accounts::ports::Message;
    use serde_json::json;

    use super::*;

    fn templates() -> EmailTemplates {
        let mut templates = EmailTemplates::default();
        let english = json!({
            "email.verify_email.subject": "Verify",
            "email.verify_email.body": "Open {link} now.",
            "email.password_changed.subject": "Changed",
            "email.password_changed.body": "Your password changed.",
            "editor.heading": "Editor",
        });
        let french = json!({
            "email.verify_email.subject": "Vérifiez",
            "email.verify_email.body": "Ouvrez {link} maintenant.",
        });
        templates.add("en".to_owned(), english.as_object().unwrap().clone());
        templates.add("fr".to_owned(), french.as_object().unwrap().clone());
        templates
    }

    fn email(language: &str, message: Message) -> Email {
        Email {
            to: "ada@example.com".to_owned(),
            language: language.to_owned(),
            message,
        }
    }

    #[test]
    fn an_email_is_rendered_in_its_language_with_its_link() {
        let link = "https://life-pixel.test/verify-email?token=abc".to_owned();
        let rendered = templates().render(&email("fr", Message::VerifyEmail { link }));
        let rendered = rendered.unwrap();
        assert_eq!(rendered.subject, "Vérifiez");
        let expected = "Ouvrez https://life-pixel.test/verify-email?token=abc maintenant.";
        assert_eq!(rendered.text, expected);
        assert!(rendered.html.contains("lang=\"fr\""));
        let anchor = "<a href=\"https://life-pixel.test/verify-email?token=abc\">";
        assert!(rendered.html.contains(anchor), "{}", rendered.html);
    }

    #[test]
    fn a_message_missing_from_a_language_is_sent_in_english() {
        let rendered = templates()
            .render(&email("fr", Message::PasswordChanged))
            .unwrap();
        assert_eq!(rendered.subject, "Changed");
        assert!(rendered.html.contains("lang=\"en\""));
        let unknown = templates().render(&email("de", Message::PasswordChanged));
        assert_eq!(unknown.unwrap().text, "Your password changed.");
    }

    #[test]
    fn a_message_missing_everywhere_is_an_error() {
        let link = "https://life-pixel.test/support".to_owned();
        let missing = templates().render(&email("en", Message::SupportReply { link }));
        assert!(
            missing
                .unwrap_err()
                .0
                .contains("email.support_reply.subject")
        );
    }

    #[test]
    fn the_repository_catalogues_have_every_email() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../i18n");
        let templates = EmailTemplates::load(&dir).unwrap();
        let link = || "https://life-pixel.test/x?token=abc".to_owned();
        for language in ["en", "fr"] {
            let messages = [
                Message::VerifyEmail { link: link() },
                Message::ResetPassword { link: link() },
                Message::PasswordChanged,
                Message::SupportReply { link: link() },
            ];
            for message in messages {
                let key = message.key();
                let wanted = format!("email.{key}.subject");
                assert!(
                    templates.messages[language].contains_key(&wanted),
                    "{language}"
                );
                let rendered = templates.render(&email(language, message)).unwrap();
                assert!(!rendered.text.contains('{'), "{language}: {key}");
            }
        }
    }
}
