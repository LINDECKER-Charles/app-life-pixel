//! A template's `{{name}}` placeholders, replaced in a single pass: a value is never read again
//! as a template.

/// What opens a placeholder.
const OPEN: &str = "{{";
/// What closes a placeholder.
const CLOSE: &str = "}}";

/// `template` with each placeholder `{{name}}` — `name` made of ASCII letters, digits and `_` —
/// replaced by `value_of(name)`; a placeholder without a value stays as it is. As in
/// `player-js`'s tests, a brace before a placeholder is text: `{{{name}}}` gives `{value}`.
pub(super) fn fill<'value>(
    template: &str,
    value_of: impl Fn(&str) -> Option<&'value str>,
) -> String {
    let mut filled = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find(OPEN) {
        let (text, from_open) = rest.split_at(start);
        filled.push_str(text);
        let replacement =
            placeholder(from_open).and_then(|(name, length)| Some((value_of(name)?, length)));
        let consumed = match replacement {
            Some((value, length)) => {
                filled.push_str(value);
                length
            }
            None => {
                filled.push_str(&from_open[..1]);
                1
            }
        };
        rest = &from_open[consumed..];
    }
    filled.push_str(rest);
    filled
}

/// The name of the placeholder `text` starts with, and the placeholder's length in bytes.
fn placeholder(text: &str) -> Option<(&str, usize)> {
    let after_open = text.strip_prefix(OPEN)?;
    let is_name_character = |character: char| character.is_ascii_alphanumeric() || character == '_';
    let name_length = after_open
        .find(|character: char| !is_name_character(character))
        .unwrap_or(after_open.len());
    let (name, after_name) = after_open.split_at(name_length);
    let is_placeholder = !name.is_empty() && after_name.starts_with(CLOSE);
    is_placeholder.then_some((name, OPEN.len() + name_length + CLOSE.len()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value_of(name: &str) -> Option<&'static str> {
        match name {
            "name" => Some("value"),
            "braces" => Some("{{name}}"),
            _ => None,
        }
    }

    #[test]
    fn placeholders_are_replaced_by_their_values() {
        assert_eq!(fill("a {{name}} b {{name}}", value_of), "a value b value");
        assert_eq!(fill("alt={{{name}}}", value_of), "alt={value}");
    }

    #[test]
    fn values_are_not_read_again_as_templates() {
        assert_eq!(fill("{{braces}}", value_of), "{{name}}");
    }

    #[test]
    fn what_is_not_a_known_placeholder_stays_as_it_is() {
        for text in [
            "{{unknown}}",
            "{{}}",
            "{{na me}}",
            "{{name",
            "{ {name}}",
            "é{{",
        ] {
            assert_eq!(fill(text, value_of), text);
        }
    }
}
