//! The minimal HTML part of an email: the text, escaped, in paragraphs, the link an anchor.

/// The argument the link replaces.
const LINK_ARGUMENT: &str = "{link}";

/// The HTML of `body`, a message of `language`, with `{link}` an anchor to `link`.
pub(super) fn render(language: &str, body: &str, link: &str) -> String {
    let link = escape(link);
    let anchor = format!("<a href=\"{link}\">{link}</a>");
    let paragraphs: String = body
        .split("\n\n")
        .map(|paragraph| {
            let lines = escape(paragraph).replace('\n', "<br>\n");
            format!("<p>{}</p>\n", lines.replace(LINK_ARGUMENT, &anchor))
        })
        .collect();
    format!(
        "<!doctype html>\n<html lang=\"{}\">\n<head><meta charset=\"utf-8\"></head>\n\
         <body>\n{paragraphs}</body>\n</html>\n",
        escape(language)
    )
}

/// `text` with the characters HTML gives a meaning escaped.
fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraphs_lines_and_the_link_become_html() {
        let html = render(
            "en",
            "Hello <you>.\n\nOpen:\n{link}",
            "https://x.test/a?b=1&c=2",
        );
        assert!(html.starts_with("<!doctype html>\n<html lang=\"en\">"));
        assert!(html.contains("<p>Hello &lt;you&gt;.</p>"));
        let anchor = "<a href=\"https://x.test/a?b=1&amp;c=2\">https://x.test/a?b=1&amp;c=2</a>";
        assert!(
            html.contains(&format!("<p>Open:<br>\n{anchor}</p>")),
            "{html}"
        );
    }
}
