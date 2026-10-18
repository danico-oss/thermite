use crate::i18n::{self, Language};
use crate::models::{Project, SceneDims, SelectedTab, SensorPos};
use crate::data;
use calamine::{open_workbook_auto, Data, Reader};
use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::Path;

pub mod chart;
pub mod config;
pub mod datagrid;
pub mod management;
pub mod map3d;
pub mod preview;
pub mod ribbon;
pub mod search;

pub use map3d::MapPreview;
pub use preview::PdfPreview;

pub struct ViewState {
    pub current_tab: SelectedTab,

    // --- Preferenze ---
    pub lang: Language,
    pub dark_mode: bool,
    pub show_settings: bool,

    // --- Pagina 1: Gestione ---
    pub selected_edit_id: Option<i64>,
    pub name_input: String,
    pub container_input: String,
    pub min_temp_input: String,
    pub max_temp_input: String,
    pub file_path_input: String,
    pub feedback_msg: Option<(String, bool)>,

    // --- Pagina 2: Ricerca ---
    pub search_query: String,

    // --- Pagina 3 & 4: Dati e Grafico ---
    pub active_project: Option<Project>,
    pub csv_headers: Vec<String>,
    pub csv_rows: Vec<Vec<String>>,
    pub csv_load_error: Option<String>,

    // --- Opzioni di visualizzazione del grafico ---
    pub show_points: bool,
    pub show_legend: bool,
    pub reset_zoom_requested: bool,

    // --- Stato globale ---
    pub status: Option<(String, bool)>,
    pub show_about: bool,

    // --- Anteprima PDF ---
    pub pdf_preview: Option<PdfPreview>,

    // --- Pagina 5: Mappa 3D dei sensori ---
    /// Progetto per cui layout e posizioni sono già stati caricati dal database.
    pub map_loaded_for: Option<i64>,
    pub map_dims: SceneDims,
    pub map_yaw: f32,
    pub map_pitch: f32,
    pub map_zoom: f32,
    pub map_positions: Vec<SensorPos>,
    pub map_selected: Option<String>,
    pub map_preview: Option<MapPreview>,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            current_tab: SelectedTab::Management,
            lang: Language::It,
            dark_mode: false,
            show_settings: false,
            selected_edit_id: None,
            name_input: String::new(),
            container_input: String::new(),
            min_temp_input: String::new(),
            max_temp_input: String::new(),
            file_path_input: String::new(),
            feedback_msg: None,
            search_query: String::new(),
            active_project: None,
            csv_headers: Vec::new(),
            csv_rows: Vec::new(),
            csv_load_error: None,
            show_points: true,
            show_legend: true,
            reset_zoom_requested: false,
            status: None,
            show_about: false,
            pdf_preview: None,
            map_loaded_for: None,
            map_dims: SceneDims::default(),
            map_yaw: 0.0,
            map_pitch: 0.0,
            map_zoom: 1.0,
            map_positions: Vec::new(),
            map_selected: None,
            map_preview: None,
        }
    }

    /// Imposta il messaggio mostrato nella barra di stato.
    pub fn set_status(&mut self, msg: impl Into<String>, is_error: bool) {
        self.status = Some((msg.into(), is_error));
    }

    pub fn clear_form(&mut self) {
        self.selected_edit_id = None;
        self.name_input.clear();
        self.container_input.clear();
        self.min_temp_input.clear();
        self.max_temp_input.clear();
        self.file_path_input.clear();
        self.feedback_msg = None;
    }

    pub fn select_project_for_operations(&mut self, p: &Project) {
        self.selected_edit_id = Some(p.id);
        self.name_input = p.name.clone();
        self.container_input = p.container_type.clone();
        self.min_temp_input = p.min_temp.to_string();
        self.max_temp_input = p.max_temp.to_string();
        self.file_path_input = p.data_file_path.clone();
        self.feedback_msg = None;
    }

    pub fn activate_project_and_load(&mut self, p: Project) {
        self.active_project = Some(p);
        self.load_data_file();
    }

    /// Apre la finestra di scelta del file dati e aggiorna il campo del form.
    pub fn pick_data_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::t(self.lang, "Seleziona il file dati dei sensori (CSV o Excel)"))
            .add_filter(i18n::t(self.lang, "File Dati"), &["csv", "txt", "xlsx", "xls"])
            .pick_file()
        {
            if let Some(path_str) = path.to_str() {
                self.file_path_input = path_str.to_string();
            }
        }
    }

    /// Carica i dati sia da file CSV/TXT che da file Excel (.xlsx / .xls)
    pub fn load_data_file(&mut self) {
        self.csv_headers.clear();
        self.csv_rows.clear();
        self.csv_load_error = None;

        let proj = match &self.active_project {
            Some(p) => p,
            None => {
                self.csv_load_error = Some(i18n::t(
                    self.lang,
                    "Nessun progetto selezionato per visualizzare i dati.",
                )
                .to_string());
                return;
            }
        };

        let path_str = &proj.data_file_path;
        let path = Path::new(path_str);
        if !path.exists() {
            self.csv_load_error = Some(format!(
                "{} {}",
                i18n::t(self.lang, "Il file dati non esiste al percorso:"),
                path_str
            ));
            return;
        }

        // Verifica di integrità: il CRC32 del file deve corrispondere a quello
        // salvato nel progetto. Se il progetto non ha un CRC registrato (creato
        // prima di questa funzionalità, o con file non leggibile al salvataggio)
        // la verifica viene saltata.
        if let Some(atteso) = &proj.data_crc32 {
            match crate::crc32::file(path) {
                Some(calcolato) => {
                    let calcolato = crate::crc32::esadecimale(calcolato);
                    if !calcolato.eq_ignore_ascii_case(atteso) {
                        self.csv_load_error = Some(format!(
                            "{} {} ({}: {}, {}: {}). {}",
                            i18n::t(self.lang, "Il file dati è cambiato dal salvataggio (CRC non corrispondente):"),
                            path_str,
                            i18n::t(self.lang, "atteso"),
                            atteso,
                            i18n::t(self.lang, "trovato"),
                            calcolato,
                            i18n::t(
                                self.lang,
                                "Per accettare il nuovo file, risalva il progetto dalla scheda Gestione.",
                            ),
                        ));
                        return;
                    }
                }
                None => {
                    self.csv_load_error = Some(format!(
                        "{} {}",
                        i18n::t(self.lang, "Impossibile leggere il file per verificare il CRC:"),
                        path_str
                    ));
                    return;
                }
            }
        }

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if extension == "xlsx" || extension == "xls" {
            let mut excel = match open_workbook_auto(path) {
                Ok(workbook) => workbook,
                Err(e) => {
                    self.csv_load_error = Some(format!(
                        "{} {}",
                        i18n::t(self.lang, "Errore apertura file Excel:"),
                        e
                    ));
                    return;
                }
            };

            let sheet_names = excel.sheet_names().to_owned();
            if let Some(sheet_name) = sheet_names.first() {
                if let Ok(range) = excel.worksheet_range(sheet_name) {
                    let mut rows_iter = range.rows();

                    // Legge la prima riga come intestazione
                    if let Some(header_row) = rows_iter.next() {
                        self.csv_headers = header_row.iter().map(cell_to_string).collect();
                    }

                    // Legge le righe successive
                    for row in rows_iter {
                        let row_vals: Vec<String> = row.iter().map(cell_to_string).collect();
                        self.csv_rows.push(row_vals);
                    }
                }
            }
        } else {
            let mut file = match File::open(path) {
                Ok(f) => f,
                Err(e) => {
                    self.csv_load_error = Some(format!(
                        "{} {}",
                        i18n::t(self.lang, "Errore apertura file:"),
                        e
                    ));
                    return;
                }
            };

            // Rileva il separatore (`,` o `;`) dalla sola prima riga, poi
            // riporta il cursore all'inizio per far leggere tutto al reader.
            let separatore = {
                let mut prima_riga = String::new();
                let _ = BufReader::new(&mut file).read_line(&mut prima_riga);
                data::detect_separator(&prima_riga)
            };
            let _ = file.seek(SeekFrom::Start(0));

            let mut rdr = csv::ReaderBuilder::new()
                .has_headers(true)
                .flexible(true)
                .delimiter(separatore)
                .from_reader(file);

            if let Ok(headers) = rdr.headers() {
                self.csv_headers = headers.iter().map(|s| s.to_string()).collect();
            }

            for record in rdr.records().flatten() {
                let row_vals: Vec<String> = record.iter().map(|s| s.to_string()).collect();
                self.csv_rows.push(row_vals);
            }
        }
    }
}

fn cell_to_string(cell: &Data) -> String {
    match cell {
        Data::String(s) => s.clone(),
        Data::Float(f) => f.to_string(),
        Data::Int(i) => i.to_string(),
        Data::Bool(b) => b.to_string(),
        Data::DateTime(d) => d.to_string(),
        Data::DateTimeIso(s) | Data::DurationIso(s) => s.clone(),
        Data::Error(e) => format!("{:?}", e),
        Data::Empty => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Project;

    /// Progetto di prova che punta a un file temporaneo.
    fn progetto(percorso: &str, crc: Option<&str>) -> Project {
        Project {
            id: 1,
            name: "Prova CRC".to_string(),
            container_type: "Container 20piedi".to_string(),
            min_temp: 0.0,
            max_temp: 30.0,
            data_file_path: percorso.to_string(),
            data_crc32: crc.map(|s| s.to_string()),
        }
    }

    /// Scrive un CSV temporaneo e ne restituisce il percorso.
    fn csv_temporaneo(nome: &str, contenuto: &[u8]) -> String {
        let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".to_string());
        let p = std::path::Path::new(&dir).join(nome);
        std::fs::write(&p, contenuto).unwrap();
        p.to_string_lossy().into_owned()
    }

    /// Con il CRC giusto i dati vengono caricati.
    #[test]
    fn crc_corretto_carica_i_dati() {
        let contenuto = b"Data,Ora,S1,S2\n2024-01-01,10:00,21.5,22.0\n2024-01-01,11:00,20.5,21.0\n";
        let percorso = csv_temporaneo("thermite_crc_ok.csv", contenuto);
        let crc = crate::crc32::file_hex(&percorso).expect("CRC calcolabile");

        let mut vs = ViewState::new();
        vs.lang = crate::i18n::Language::It;
        vs.active_project = Some(progetto(&percorso, Some(&crc)));
        vs.load_data_file();

        assert_eq!(vs.csv_load_error, None, "errore: {:?}", vs.csv_load_error);
        assert_eq!(vs.csv_headers, vec!["Data", "Ora", "S1", "S2"]);
        assert_eq!(vs.csv_rows.len(), 2);
        let _ = std::fs::remove_file(&percorso);
    }

    /// Se il file cambia dopo il salvataggio, il caricamento viene bloccato.
    #[test]
    fn crc_diverso_blocca_il_caricamento() {
        let originale = b"Data,Ora,S1\n2024-01-01,10:00,21.5\n";
        let percorso = csv_temporaneo("thermite_crc_ko.csv", originale);
        let crc = crate::crc32::file_hex(&percorso).expect("CRC calcolabile");

        let mut vs = ViewState::new();
        vs.lang = crate::i18n::Language::It;
        vs.active_project = Some(progetto(&percorso, Some(&crc)));
        vs.load_data_file();
        assert_eq!(vs.csv_load_error, None, "prima: {:?}", vs.csv_load_error);

        // Il file viene sostituito con contenuto diverso.
        let nuovo = b"Data,Ora,S1\n2024-01-01,10:00,99.9\n";
        std::fs::write(&percorso, nuovo).unwrap();
        vs.load_data_file();

        let err = vs.csv_load_error.expect("doveva rifiutare il file modificato");
        assert!(
            err.contains("CRC"),
            "il messaggio deve parlare di CRC: {err}"
        );
        // Nessun dato vecchio o nuovo deve essere stato caricato.
        assert!(vs.csv_headers.is_empty());
        assert!(vs.csv_rows.is_empty());
        let _ = std::fs::remove_file(&percorso);
    }

    /// L'archivio di dati fasulli consegnato con il progetto deve caricarsi
    /// per intero: 25 registrazioni orarie, 4 sensori, valori tutti fra 2 e 8.
    ///
    /// Verifica anche l'ora limite `24:00:00` (mezzanotte di fine giornata,
    /// valida in ISO 8601): se il parser la rifiutasse, l'ultima riga
    /// sparirebbe in silenzio da grafico e mappa 3D.
    #[test]
    fn archivio_dati_fasulli_si_carica_per_intero() {
        let percorso = "data/dati_fasulli_17-10-2026.csv";
        assert!(
            std::path::Path::new(percorso).exists(),
            "manca l'archivio di prova: {percorso}"
        );

        let mut vs = ViewState::new();
        vs.lang = crate::i18n::Language::It;
        vs.active_project = Some(progetto(percorso, None));
        vs.load_data_file();

        assert_eq!(vs.csv_load_error, None, "errore: {:?}", vs.csv_load_error);
        assert_eq!(
            vs.csv_headers,
            vec!["Data", "Ora", "Sensore_1", "Sensore_2", "Sensore_3", "Sensore_4"]
        );
        assert_eq!(vs.csv_rows.len(), 25, "attese 25 registrazioni orarie");

        let series = crate::data::parse_series(&vs.csv_headers, &vs.csv_rows);
        assert_eq!(series.len(), 4, "attesi 4 sensori");
        for s in &series {
            assert_eq!(
                s.points.len(),
                25,
                "sensore {}: qualche ora non è stata convertita",
                s.name
            );
            for p in &s.points {
                assert!(
                    (2.0..=8.0).contains(&p[1]),
                    "sensore {}: valore {} fuori dall'intervallo 2-8",
                    s.name,
                    p[1]
                );
            }
        }

        // La riga 24:00:00 è la mezzanotte di fine giornata: deve cadere 24 ore
        // dopo la prima, non sovrapporsi ad essa.
        let prima = series[0].points.first().expect("serie vuota");
        let ultima = series[0].points.last().expect("serie vuota");
        assert_eq!(
            crate::data::format_timestamp(prima[0]),
            "17/10/2026 00:00"
        );
        assert_eq!(
            crate::data::format_timestamp(ultima[0]),
            "18/10/2026 00:00",
            "la riga 24:00:00 non è stata interpretata come fine giornata"
        );
        assert_eq!(
            (ultima[0] - prima[0]) / 3600.0,
            24.0,
            "le 25 registrazioni devono coprire 24 ore"
        );
    }

    /// Un file separato da `;` con virgola decimale viene letto correttamente:
    /// il separatore è rilevato dalla prima riga, e i valori restano numerici.
    #[test]
    fn file_con_separatore_punto_e_virgola() {
        let contenuto = "Data;Ora;Sensore_1;Sensore_2\n2024-01-01;10:00;21,5;22,0\n2024-01-01;11:00;20,5;21,0\n";
        let percorso = csv_temporaneo("thermite_semicolon.csv", contenuto.as_bytes());

        let mut vs = ViewState::new();
        vs.lang = crate::i18n::Language::It;
        vs.active_project = Some(progetto(&percorso, None));
        vs.load_data_file();

        assert_eq!(vs.csv_load_error, None, "errore: {:?}", vs.csv_load_error);
        assert_eq!(vs.csv_headers, vec!["Data", "Ora", "Sensore_1", "Sensore_2"]);
        assert_eq!(vs.csv_rows.len(), 2);
        assert_eq!(vs.csv_rows[0], vec!["2024-01-01", "10:00", "21,5", "22,0"]);

        // La virgola decimale deve produrre numeri veri, non celle spezzate.
        let series = crate::data::parse_series(&vs.csv_headers, &vs.csv_rows);
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].name, "Sensore_1");
        assert_eq!(series[0].points.len(), 2);
        assert_eq!(series[0].points[0][1], 21.5, "virgola decimale non convertita");
        assert_eq!(series[1].points[1][1], 21.0);

        let _ = std::fs::remove_file(&percorso);
    }

    /// Un progetto senza CRC registrato continua a caricare (compatibilità).
    #[test]
    fn senza_crc_registrato_carica_comunque() {
        let contenuto = b"Data,Ora,S1\n2024-01-01,10:00,21.5\n";
        let percorso = csv_temporaneo("thermite_crc_null.csv", contenuto);

        let mut vs = ViewState::new();
        vs.lang = crate::i18n::Language::It;
        vs.active_project = Some(progetto(&percorso, None));
        vs.load_data_file();

        assert_eq!(vs.csv_load_error, None, "errore: {:?}", vs.csv_load_error);
        assert_eq!(vs.csv_rows.len(), 1);
        let _ = std::fs::remove_file(&percorso);
    }

    /// Salva il progetto: il CRC del file finisce nel database.
    #[test]
    fn salvataggio_scrive_il_crc_nel_database() {
        let contenuto = b"Data,Ora,S1\n2024-01-01,10:00,21.5\n";
        let percorso = csv_temporaneo("thermite_crc_db.csv", contenuto);

        let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".to_string());
        let db_path = std::path::Path::new(&dir).join("thermite_crc_prova.db");
        let _ = std::fs::remove_file(&db_path);
        let db = crate::models::Database::open_at(&db_path.to_string_lossy());

        let mut vs = ViewState::new();
        vs.lang = crate::i18n::Language::It;
        vs.name_input = "Prova".to_string();
        vs.container_input = "Container 20piedi".to_string();
        vs.min_temp_input = "0".to_string();
        vs.max_temp_input = "30".to_string();
        vs.file_path_input = percorso.clone();
        let mut reload = false;
        vs.try_save(&db.conn, &mut reload);

        let progetti = db.load_all_projects();
        assert_eq!(progetti.len(), 1);
        let atteso = crate::crc32::file_hex(&percorso).expect("CRC calcolabile");
        assert_eq!(progetti[0].data_crc32.as_deref(), Some(atteso.as_str()));

        // Il progetto riletto dal database carica i dati senza errori.
        let mut vs2 = ViewState::new();
        vs2.lang = crate::i18n::Language::It;
        vs2.active_project = Some(progetti[0].clone());
        vs2.load_data_file();
        assert_eq!(vs2.csv_load_error, None, "errore: {:?}", vs2.csv_load_error);

        let _ = std::fs::remove_file(&percorso);
        let _ = std::fs::remove_file(&db_path);
    }
}
