//! Escaping a value for the language of the snippet it goes into.

use std::fmt::Write as _;

/// A value inside a double-quoted HTML attribute — in a page, a Vue template or a JSX string
/// attribute, which all decode character references: the characters that could close the
/// attribute or open markup become references.
pub(super) fn html_attribute(value: &str) -> String {
    escape_with(value, html_reference)
}

/// A value inside a double-quoted attribute of an Angular template written as a JavaScript
/// template literal: [`html_attribute`], plus references for what would end the literal or
/// interpolate into it — `` ` ``, `$`, `\` — and for the braces of Angular's `{{ }}`.
pub(super) fn angular_attribute(value: &str) -> String {
    escape_with(value, |character| {
        html_reference(character).or_else(|| template_literal_reference(character))
    })
}

/// A value as a double-quoted JavaScript string literal: `\` and `"` escaped, line breaks and
/// control characters written as escapes, so the literal stays on one line.
pub(super) fn javascript_string(value: &str) -> String {
    let mut literal = String::with_capacity(value.len() + 2);
    literal.push('"');
    for character in value.chars() {
        match character {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\t' => literal.push_str("\\t"),
            '\u{2028}' | '\u{2029}' => write_unicode_escape(&mut literal, character),
            control if control.is_control() => write_unicode_escape(&mut literal, control),
            _ => literal.push(character),
        }
    }
    literal.push('"');
    literal
}

fn html_reference(character: char) -> Option<&'static str> {
    match character {
        '&' => Some("&amp;"),
        '"' => Some("&quot;"),
        '\'' => Some("&#39;"),
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        _ => None,
    }
}

fn template_literal_reference(character: char) -> Option<&'static str> {
    match character {
        '`' => Some("&#96;"),
        '$' => Some("&#36;"),
        '\\' => Some("&#92;"),
        '{' => Some("&#123;"),
        '}' => Some("&#125;"),
        _ => None,
    }
}

/// `value` with each character `reference` names replaced by its reference.
fn escape_with(value: &str, reference: impl Fn(char) -> Option<&'static str>) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match reference(character) {
            Some(reference) => escaped.push_str(reference),
            None => escaped.push(character),
        }
    }
    escaped
}

/// `\uXXXX`: every character it is used for is in the Basic Multilingual Plane.
fn write_unicode_escape(literal: &mut String, character: char) {
    // Writing into a `String` cannot fail.
    let _ = write!(literal, "\\u{:04x}", u32::from(character));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_attributes_escape_what_closes_them_or_opens_markup() {
        assert_eq!(
            html_attribute(r#"Tom & "Jerry" <3 'n' {{x}}"#),
            "Tom &amp; &quot;Jerry&quot; &lt;3 &#39;n&#39; {{x}}"
        );
        assert_eq!(html_attribute("/a.wasm?v=1&t=2"), "/a.wasm?v=1&amp;t=2");
    }

    #[test]
    fn angular_attributes_also_escape_the_template_literal_and_interpolation() {
        assert_eq!(
            angular_attribute(r#"`${a}` {{b}} \n & ""#),
            "&#96;&#36;&#123;a&#125;&#96; &#123;&#123;b&#125;&#125; &#92;n &amp; &quot;"
        );
    }

    #[test]
    fn javascript_strings_escape_quotes_backslashes_and_line_breaks() {
        assert_eq!(javascript_string("The mascot"), r#""The mascot""#);
        assert_eq!(
            javascript_string("\"a\\b\"\n\t\u{7}\u{2028}é"),
            r#""\"a\\b\"\n\t\u0007\u2028é""#
        );
    }
}
