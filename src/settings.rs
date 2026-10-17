//! Persistenza minimale delle preferenze dell'app (tema e lingua) su un piccolo
//! file di testo nella cartella di lavoro, accanto al database.

use crate::i18n::Language;
use std::path::PathBuf;

const FILE_NAME: &str = "thermite_settings.txt";

#[derive(Clone, Debug)]
pub struct Settings {
    /// `true` = tema scuro, `false` = tema chiaro.
    pub dark_mode: bool,
    /// Lingua dell'interfaccia.
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

/// Legge le preferenze dal file; se manca o è corrotto usa i valori predefiniti.
pub fn load() -> Settings {
    match std::fs::read_to_string(path()) {
        Ok(content) => parse(&content),
        Err(_) => Settings::default(),
    }
}

/// Interpreta il contenuto del file di preferenze (formato `chiave=valore`).
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

/// Salva le preferenze; eventuali errori di scrittura vengono ignorati.
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
        // righe ignorate / chiavi sconosciute
        assert!(!parse("# commento\naltra=cosa\ndark_mode=0").dark_mode);
        // contenuto vuoto / malformato -> predefinito
        assert!(!parse("").dark_mode);
        assert!(!parse("senza_uguale").dark_mode);
    }

    #[test]
    fn interpreta_la_lingua() {
        assert_eq!(parse("lang=pt-BR").lang, Language::PtBr);
        assert_eq!(parse("lingua=EN").lang, Language::En);
        assert_eq!(parse("language=it").lang, Language::It);
        // valore sconosciuto -> resta il predefinito
        assert_eq!(parse("lang=xx").lang, Language::It);
    }
}
