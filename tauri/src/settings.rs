//! The person's choices, kept in `settings.json` in the app's configuration folder: the library
//! folder, the language, the theme and the motion.

use std::fs;
use std::io::{self, Write as _};
use std::path::Path;

use serde::{Deserialize, Serialize};

/// The file the settings are kept in, inside the app's configuration folder.
pub const SETTINGS_FILE: &str = "settings.json";

/// The longest language code accepted: a BCP 47 tag of a few subtags.
const LANGUAGE_MAX_CHARS: usize = 35;

/// The colours: the system's scheme, or a set chosen by hand.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// The system's scheme.
    #[default]
    System,
    /// Light colours.
    Light,
    /// Dark colours.
    Dark,
}

/// Motion follows the system's setting, or is reduced whatever the system says.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Motion {
    /// The system's setting.
    #[default]
    System,
    /// Reduced motion.
    Reduce,
}

/// The settings, as `settings.json` holds them and `settings_get` answers them. A missing field
/// takes its default, so that an older file still reads.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    /// The library folder; `None` for the default one.
    pub library_path: Option<String>,
    /// The interface's language code; `None` until the person chooses one.
    pub language: Option<String>,
    /// The colours.
    pub theme: Theme,
    /// The motion.
    pub motion: Motion,
}

impl Settings {
    /// The settings in `file`: the defaults when it does not exist yet, or does not read — a
    /// damaged file never keeps the app from starting.
    #[must_use]
    pub fn read(file: &Path) -> Self {
        let text = match fs::read(file) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Self::default(),
            Err(error) => {
                tracing::warn!(path = %file.display(), %error, "settings not read");
                return Self::default();
            }
        };
        serde_json::from_slice(&text).unwrap_or_else(|error| {
            tracing::warn!(path = %file.display(), %error, "settings ignored");
            Self::default()
        })
    }

    /// Writes the settings to `file`, creating its folder: a temporary file beside it, renamed
    /// over it, so that a crash never leaves half a file.
    ///
    /// # Errors
    ///
    /// When the folder or the file cannot be written.
    pub fn write(&self, file: &Path) -> io::Result<()> {
        let folder = file.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(folder)?;
        let mut text = serde_json::to_vec_pretty(self)?;
        text.push(b'\n');
        let mut temporary = tempfile::NamedTempFile::new_in(folder)?;
        temporary.write_all(&text)?;
        temporary.as_file().sync_all()?;
        temporary.persist(file).map_err(|error| error.error)?;
        Ok(())
    }

    /// Whether the language, when set, looks like a language code: 1 to 35 ASCII letters, digits
    /// and hyphens. Which languages exist is the catalogues' business.
    #[must_use]
    pub fn has_valid_language(&self) -> bool {
        self.language.as_deref().is_none_or(|code| {
            (1..=LANGUAGE_MAX_CHARS).contains(&code.len())
                && code.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        })
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_settings_are_written_then_read_back() {
        let folder = tempfile::tempdir().unwrap();
        let file = folder.path().join("config").join(SETTINGS_FILE);
        let settings = Settings {
            library_path: Some("/tmp/library".to_owned()),
            language: Some("fr".to_owned()),
            theme: Theme::Dark,
            motion: Motion::Reduce,
        };
        settings.write(&file).unwrap();
        assert_eq!(Settings::read(&file), settings);
        let json: serde_json::Value = serde_json::from_slice(&fs::read(&file).unwrap()).unwrap();
        let expected = json!({
            "libraryPath": "/tmp/library", "language": "fr", "theme": "dark", "motion": "reduce"
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn a_missing_or_damaged_file_gives_the_defaults() {
        let folder = tempfile::tempdir().unwrap();
        let file = folder.path().join(SETTINGS_FILE);
        assert_eq!(Settings::read(&file), Settings::default());
        fs::write(&file, b"{ not json").unwrap();
        assert_eq!(Settings::read(&file), Settings::default());
        fs::write(&file, br#"{"theme": "light"}"#).unwrap();
        assert_eq!(Settings::read(&file).theme, Theme::Light);
    }

    #[test]
    fn a_language_is_a_short_code() {
        let with = |code: &str| Settings {
            language: Some(code.to_owned()),
            ..Settings::default()
        };
        assert!(Settings::default().has_valid_language());
        assert!(with("pt-BR").has_valid_language());
        assert!(!with("").has_valid_language());
        assert!(!with("../en").has_valid_language());
        assert!(!with(&"a".repeat(36)).has_valid_language());
    }
}
