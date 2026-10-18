//! Minimal persistence of the app preferences (theme and language) to a small
//! text file in the working directory, next to the database.

use crate::i18n::Language;
use std::path::PathBuf;

const FILE_NAME: &str = "thermite_settings.txt";

#[derive(Clone, Debug)]
pub struct Settings {
    /// `true` = dark theme, `false` = light theme.
    pub dark_mode: bool,
    /// Interface language.
    pub lang: Language,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            dark_mode: false,
            lang: Language::It,
        }
    }
}

fn path() -> PathBuf {
    PathBuf::from(FILE_NAME)
}

/// Reads the preferences from the file; if it is missing or corrupt, uses the defaults.
pub fn load() -> Settings {
    match std::fs::read_to_string(path()) {
        Ok(content) => parse(&content),
        Err(_) => Settings::default(),
    }
}

/// Interprets the content of the preferences file (format `key=value`).
fn parse(content: &str) -> Settings {
    let mut settings = Settings::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with("//") {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "dark_mode" | "tema_scuro" => {
                settings.dark_mode = matches!(
                    value.trim().to_lowercase().as_str(),
                    "true" | "1" | "yes" | "si" | "sì" | "on"
                );
            }
            "lang" | "lingua" | "language" => {
                if let Some(lang) = Language::from_code(value) {
                    settings.lang = lang;
                }
            }
            _ => {}
        }
    }

    settings
}

/// Saves the preferences; any write errors are ignored.
pub fn save(settings: &Settings) {
    let content = format!(
        "# Preferenze di Thermite (modificabili a mano)\n\
         # dark_mode: true = tema scuro, false = tema chiaro\n\
         # lang: it | pt-BR | en\n\
         dark_mode={}\n\
         lang={}\n",
        settings.dark_mode,
        settings.lang.code()
    );
    let _ = std::fs::write(path(), content);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predefinito_e_chiaro() {
        assert!(!Settings::default().dark_mode);
        assert_eq!(Settings::default().lang, Language::It);
    }

    #[test]
    fn interpreta_le_preferenze() {
        assert!(parse("dark_mode=true").dark_mode);
        assert!(parse("dark_mode = 1").dark_mode);
        assert!(parse("tema_scuro=Sì").dark_mode);
        assert!(!parse("dark_mode=false").dark_mode);
        assert!(!parse("dark_mode=no").dark_mode);
        // ignored lines / unknown keys
        assert!(!parse("# commento\naltra=cosa\ndark_mode=0").dark_mode);
        // empty / malformed content -> default
        assert!(!parse("").dark_mode);
        assert!(!parse("senza_uguale").dark_mode);
    }

    #[test]
    fn interpreta_la_lingua() {
        assert_eq!(parse("lang=pt-BR").lang, Language::PtBr);
        assert_eq!(parse("lingua=EN").lang, Language::En);
        assert_eq!(parse("language=it").lang, Language::It);
        // unknown value -> the default is kept
        assert_eq!(parse("lang=xx").lang, Language::It);
    }
}
