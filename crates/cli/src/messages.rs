//! The `cli.` and `errors.` messages: the catalogues embedded at build time, the caller's
//! language, and rendering a key's `{placeholder}`s from named parameters.

use std::env;

use life_pixel_service::CodedError;
use serde_json::{Map, Value};

/// `i18n/en.json`, embedded at build time.
const ENGLISH: &str = include_str!("../../../i18n/en.json");
/// `i18n/fr.json`, embedded at build time.
const FRENCH: &str = include_str!("../../../i18n/fr.json");
/// The variables consulted, in precedence order.
const LOCALE_VARIABLES: [&str; 3] = ["LC_ALL", "LC_MESSAGES", "LANG"];

/// A language the CLI's own output speaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    /// The fallback, and the language of a locale that does not start with `fr`.
    English,
    /// `LC_ALL`, `LC_MESSAGES` or `LANG` starts with `fr`.
    French,
}

impl Language {
    /// The first of `LC_ALL`, `LC_MESSAGES` and `LANG` that is set, read for a locale starting
    /// with `fr`.
    #[must_use]
    pub fn from_env() -> Self {
        let values = LOCALE_VARIABLES.map(|name| env::var(name).ok());
        Self::from_locale(values.into_iter().flatten().find(|value| !value.is_empty()))
    }

    fn from_locale(locale: Option<String>) -> Self {
        let is_french = locale.is_some_and(|locale| locale.to_lowercase().starts_with("fr"));
        if is_french {
            Self::French
        } else {
            Self::English
        }
    }

    fn catalogue_text(self) -> &'static str {
        match self {
            Self::English => ENGLISH,
            Self::French => FRENCH,
        }
    }
}

/// The `cli.` and `errors.` messages of one language, English kept alongside as the fallback.
pub struct Messages {
    catalogue: Map<String, Value>,
    fallback: Map<String, Value>,
}

impl Messages {
    /// Loads `language`'s catalogue; English twice when `language` already is English.
    #[must_use]
    pub fn load(language: Language) -> Self {
        let fallback = parse(Language::English.catalogue_text());
        let catalogue = match language {
            Language::English => fallback.clone(),
            Language::French => parse(language.catalogue_text()),
        };
        Self {
            catalogue,
            fallback,
        }
    }

    /// The message of `key`, its `{name}` placeholders replaced from `params`; falls back to
    /// English, then to the key itself.
    #[must_use]
    pub fn text(&self, key: &str, params: &[(&str, &str)]) -> String {
        let template = self.catalogue.get(key).or_else(|| self.fallback.get(key));
        let template = template.and_then(Value::as_str).unwrap_or(key);
        substitute(template, params)
    }

    /// The message of a coded error: `errors.<code>`, its parameters read as text.
    #[must_use]
    pub fn error(&self, error: &CodedError) -> String {
        let key = format!("errors.{}", error.code);
        let params: Vec<(String, String)> = error
            .params
            .iter()
            .map(|(name, value)| (name.clone(), display(value)))
            .collect();
        let params: Vec<(&str, &str)> = params
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
            .collect();
        self.text(&key, &params)
    }
}

/// `catalogue`'s valid JSON as an object, or an empty one: the repository's own files always
/// parse, checked by `npm run i18n:check`.
fn parse(catalogue: &str) -> Map<String, Value> {
    let value: Value = serde_json::from_str(catalogue).unwrap_or_default();
    value.as_object().cloned().unwrap_or_default()
}

/// `value` as the text a message's placeholder shows: a string as it is, anything else as JSON.
fn display(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}

/// `template`, each `{name}` or `{name, …}` replaced by the value `params` gives `name`, left as
/// it is when `params` has none.
fn substitute(template: &str, params: &[(&str, &str)]) -> String {
    let mut result = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        result.push_str(&rest[..start]);
        let Some(length) = rest[start..].find('}') else {
            result.push_str(&rest[start..]);
            return result;
        };
        let inside = &rest[start + 1..start + length];
        let whole = &rest[start..=start + length];
        let name = inside
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .next();
        let value = name.and_then(|name| params.iter().find(|(key, _)| *key == name));
        result.push_str(value.map_or(whole, |(_, value)| value));
        rest = &rest[start + length + 1..];
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_locale_variables_are_read_in_order_for_a_french_prefix() {
        assert_eq!(Language::from_locale(None), Language::English);
        assert_eq!(
            Language::from_locale(Some("en_US.UTF-8".to_owned())),
            Language::English
        );
        assert_eq!(
            Language::from_locale(Some("fr_FR.UTF-8".to_owned())),
            Language::French
        );
        assert_eq!(
            Language::from_locale(Some("FR".to_owned())),
            Language::French
        );
    }

    #[test]
    fn a_placeholder_is_replaced_and_an_icu_annotation_is_dropped() {
        assert_eq!(substitute("Hello {name}", &[("name", "Ada")]), "Hello Ada");
        assert_eq!(substitute("{max, number} max", &[("max", "10")]), "10 max");
        assert_eq!(substitute("{unknown}", &[]), "{unknown}");
    }

    #[test]
    fn a_missing_key_falls_back_to_english_then_to_itself() {
        let messages = Messages {
            catalogue: Map::new(),
            fallback: Map::from_iter([("cli.hello".to_owned(), json!("Hello"))]),
        };
        assert_eq!(messages.text("cli.hello", &[]), "Hello");
        assert_eq!(messages.text("cli.missing", &[]), "cli.missing");
    }

    #[test]
    fn an_error_s_parameters_render_as_text() {
        let messages = Messages::load(Language::English);
        let mut params = Map::new();
        params.insert("path".to_owned(), json!("/tmp/lib"));
        let error = CodedError {
            code: "library.unavailable",
            params,
        };
        assert_eq!(
            messages.error(&error),
            "The library folder /tmp/lib cannot be opened. Check that it exists and that you \
             can write to it."
        );
    }
}
