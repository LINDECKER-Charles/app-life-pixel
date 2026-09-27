//! The value of each placeholder of the templates, escaped for the snippet's language.

use super::escape::{angular_attribute, html_attribute, javascript_string};
use super::{Framework, SnippetInput};

/// What follows the PascalCase stem in a component's name.
const CLASS_SUFFIX: &str = "Animation";
/// What precedes a PascalCase stem that starts with a digit, which no identifier can.
const DIGIT_PREFIX: &str = "Pixel";

/// The placeholders' values for one snippet.
pub(super) struct Values {
    loader: String,
    src: String,
    tag_attribute: String,
    alt: String,
    alt_expression: String,
    class_name: String,
}

impl Values {
    /// The values of `input`, each attribute value escaped for `framework`.
    pub fn new(framework: Framework, input: &SnippetInput) -> Self {
        let attribute = match framework {
            Framework::Angular => angular_attribute,
            Framework::Html | Framework::React | Framework::Vue => html_attribute,
        };
        let tag = input.tag.as_deref().filter(|tag| !tag.is_empty());
        Self {
            loader: attribute(&input.loader),
            src: attribute(&input.src),
            tag_attribute: tag
                .map_or_else(String::new, |tag| format!(" tag=\"{}\"", attribute(tag))),
            alt: attribute(&input.alt),
            alt_expression: javascript_string(&input.alt),
            class_name: class_name(&input.file_stem),
        }
    }

    /// The value of the placeholder `name`, as the templates of `player-js/snippets/` name it.
    pub fn get(&self, name: &str) -> Option<&str> {
        let value = match name {
            "loader" => &self.loader,
            "src" => &self.src,
            "tagAttribute" => &self.tag_attribute,
            "alt" => &self.alt,
            "altExpression" => &self.alt_expression,
            "className" => &self.class_name,
            _ => return None,
        };
        Some(value)
    }
}

/// The stem in PascalCase followed by `Animation`: `mascot-idle` gives `MascotIdleAnimation`.
/// Only ASCII letters and digits are kept, and a name that would start with a digit starts with
/// `Pixel`: `2-cats` gives `Pixel2CatsAnimation`.
fn class_name(stem: &str) -> String {
    let words = stem.split(|character: char| !character.is_ascii_alphanumeric());
    let mut name: String = words.map(capitalized).collect();
    if name.starts_with(|character: char| character.is_ascii_digit()) {
        name.insert_str(0, DIGIT_PREFIX);
    }
    name.push_str(CLASS_SUFFIX);
    name
}

/// `word` with its first letter in uppercase.
fn capitalized(word: &str) -> String {
    let mut characters = word.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_ascii_uppercase().to_string() + characters.as_str()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn class_names_are_the_stem_in_pascal_case_followed_by_animation() {
        assert_eq!(class_name("mascot"), "MascotAnimation");
        assert_eq!(class_name("mascot-idle-2"), "MascotIdle2Animation");
        assert_eq!(class_name("hero--run_"), "HeroRunAnimation");
        assert_eq!(class_name("2-cats"), "Pixel2CatsAnimation");
        assert_eq!(class_name("été"), "TAnimation");
        assert_eq!(class_name(""), "Animation");
    }
}
