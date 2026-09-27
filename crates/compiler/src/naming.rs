//! The names of exported files, taken from the animation's title.

/// The stem when the title keeps no ASCII letter or digit.
const FALLBACK_STEM: &str = "animation";
/// What joins the words of a stem.
const SEPARATOR: &str = "-";

/// The title lowercased, its ASCII letters and digits kept and everything between them turned
/// into a single `-`: `"Mascot — Idle 2"` gives `mascot-idle-2`, and a title with nothing left
/// gives `animation`.
#[must_use]
pub fn file_stem(title: &str) -> String {
    let words: Vec<&str> = title
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    if words.is_empty() {
        return FALLBACK_STEM.to_owned();
    }
    words.join(SEPARATOR).to_ascii_lowercase()
}

/// The name of frame `position` in a zip of `frame_count` frames: the stem, `-`, and the position
/// padded with zeros to as many digits as the last position needs — `mascot-07.png` of 12.
pub(crate) fn frame_file_name(stem: &str, position: usize, frame_count: usize) -> String {
    let digits = frame_count.saturating_sub(1).to_string().len();
    format!("{stem}-{position:0digits$}.png")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_keeps_its_ascii_words_in_lowercase() {
        assert_eq!(file_stem("Mascot"), "mascot");
        assert_eq!(file_stem("  Mascot — Idle 2!"), "mascot-idle-2");
        assert_eq!(file_stem("Été à Paris"), "t-paris");
        assert_eq!(file_stem("a__b..c"), "a-b-c");
    }

    #[test]
    fn a_title_without_ascii_letters_or_digits_falls_back_to_animation() {
        assert_eq!(file_stem("日本"), "animation");
        assert_eq!(file_stem("—"), "animation");
        assert_eq!(file_stem(""), "animation");
    }

    #[test]
    fn frame_names_take_as_many_digits_as_the_last_position() {
        assert_eq!(frame_file_name("mascot", 0, 1), "mascot-0.png");
        assert_eq!(frame_file_name("mascot", 9, 10), "mascot-9.png");
        assert_eq!(frame_file_name("mascot", 7, 12), "mascot-07.png");
        assert_eq!(frame_file_name("mascot", 0, 1_024), "mascot-0000.png");
    }
}
