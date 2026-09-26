//! What the app attaches to a request without asking.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::support::SupportError;

/// The most characters of the app version, the platform and the language.
pub const CONTEXT_FIELD_MAX_CHARS: usize = 64;
/// The most characters of the screen's route template.
pub const SCREEN_MAX_CHARS: usize = 200;

/// The segment a route template may end with to match anything: the not-found page's.
const WILDCARD_SEGMENT: &str = "**";
/// The prefix of a route template's parameter segment, `:animationId` for example.
const PARAMETER_PREFIX: char = ':';

/// The context of a request: the app's version, its platform and language, and the screen the
/// person was on, as the route's template — `/editor/:animationId` —, never a URL with ids.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SupportContext {
    /// The app's version, `0.1.0` for example.
    pub app_version: String,
    /// The app's platform: `web`, `android`…
    pub platform: String,
    /// The interface's language code.
    pub language: String,
    /// The route template of the screen the person was on.
    pub screen: String,
}

impl SupportContext {
    /// The context of `json`, as the app sends it.
    ///
    /// # Errors
    ///
    /// `request.malformed` when it does not parse, a field is empty or too long, or the screen is
    /// not a route template.
    pub fn parse(json: &[u8]) -> Result<Self, SupportError> {
        let context: Self = serde_json::from_slice(json).map_err(|_| SupportError::Malformed)?;
        let short = [&context.app_version, &context.platform, &context.language];
        let fields_fit = short
            .iter()
            .all(|field| fits(field, CONTEXT_FIELD_MAX_CHARS));
        if !fields_fit || !fits(&context.screen, SCREEN_MAX_CHARS) || !is_template(&context.screen)
        {
            return Err(SupportError::Malformed);
        }
        Ok(context)
    }
}

/// Whether `text` holds 1 to `max` characters.
fn fits(text: &str, max: usize) -> bool {
    (1..=max).contains(&text.chars().count())
}

/// Whether `screen` is a route template: `/` then segments of lowercase letters, digits and
/// hyphens, `:parameter`s, or `**` — never an id.
fn is_template(screen: &str) -> bool {
    let Some(path) = screen.strip_prefix('/') else {
        return false;
    };
    path.is_empty() || path.split('/').all(is_template_segment)
}

fn is_template_segment(segment: &str) -> bool {
    if segment == WILDCARD_SEGMENT {
        return true;
    }
    if let Some(name) = segment.strip_prefix(PARAMETER_PREFIX) {
        return !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric());
    }
    let is_word = segment
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    let is_number = segment.chars().all(|c| c.is_ascii_digit());
    !segment.is_empty() && is_word && !is_number && Uuid::parse_str(segment).is_err()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(screen: &str) -> String {
        format!(r#"{{"appVersion":"0.1.0","platform":"web","language":"en","screen":"{screen}"}}"#)
    }

    #[test]
    fn a_route_template_is_a_screen() {
        for screen in [
            "/",
            "/editor/:animationId",
            "/reset-password/confirm",
            "/**",
        ] {
            let parsed = SupportContext::parse(context(screen).as_bytes()).unwrap();
            assert_eq!(parsed.screen, screen);
        }
    }

    #[test]
    fn a_url_with_ids_is_not_a_screen() {
        let screens = [
            "editor",
            "/editor/0190f1c2-7a4e-7b3c-9d2e-1f2a3b4c5d6e",
            "/library/projects/42",
            "/editor?id=1",
            "/Editor",
            "//",
            "/:",
        ];
        for screen in screens {
            let parsed = SupportContext::parse(context(screen).as_bytes());
            assert_eq!(parsed, Err(SupportError::Malformed), "{screen}");
        }
    }

    #[test]
    fn a_context_needs_every_field_and_no_other() {
        let missing = r#"{"appVersion":"0.1.0","platform":"web","screen":"/"}"#;
        let extra = r#"{"appVersion":"1","platform":"web","language":"en","screen":"/","url":"x"}"#;
        let empty = r#"{"appVersion":"","platform":"web","language":"en","screen":"/"}"#;
        for json in [missing, extra, empty, "not json"] {
            let parsed = SupportContext::parse(json.as_bytes());
            assert_eq!(parsed, Err(SupportError::Malformed), "{json}");
        }
    }
}
