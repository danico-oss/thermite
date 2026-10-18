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

    // --- Preferences ---
    pub lang: Language,
    pub dark_mode: bool,
    pub show_settings: bool,

    // --- Page 1: Management ---
    pub selected_edit_id: Option<i64>,
    pub name_input: String,
    pub container_input: String,
    pub min_temp_input: String,
    pub max_temp_input: String,
    pub file_path_input: String,
    pub feedback_msg: Option<(String, bool)>,

    // --- Page 2: Search ---
    pub search_query: String,

    // --- Page 3 & 4: Data and Chart ---
    pub active_project: Option<Project>,
    pub csv_headers: Vec<String>,
    pub csv_rows: Vec<Vec<String>>,
    pub csv_load_error: Option<String>,

    // --- Chart display options ---
    pub show_points: bool,
    pub show_legend: bool,
    pub reset_zoom_requested: bool,

    // --- Global state ---
    pub status: Option<(String, bool)>,
    pub show_about: bool,

    // --- PDF preview ---
    pub pdf_preview: Option<PdfPreview>,

    // --- Page 5: 3D sensor map ---
    /// Project for which the layout and positions have already been loaded from the database.
    pub map_loaded_for: Option<i64>,
    pub map_dims: SceneDims,
    pub map_yaw: f32,
    pub map_pitch: f32,
    pub map_zoom: f32,
    pub map_positions: Vec<SensorPos>,
    pub map_selected: Option<String>,
    /// Sensor currently being dragged on the canvas (it vanishes when the
    /// button is released). While active, the canvas does not rotate the scene.
    pub map_dragging: Option<String>,
    /// Currently placed sensors hidden on the diagram, by name.
    /// Session-only: it is not saved anywhere and is reset when switching to
    /// another project.
    pub map_hidden: std::collections::BTreeSet<String>,
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
            map_dragging: None,
            map_hidden: std::collections::BTreeSet::new(),
            map_preview: None,
        }
    }

    /// Sets the message shown in the status bar.
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

    /// Opens the data file chooser dialog and updates the form field.
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

    /// Loads the data from either CSV/TXT files or Excel files (.xlsx / .xls)
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

        // Integrity check: the file's CRC32 must match the one saved in the
        // project. If the project has no recorded CRC (created before this
        // feature, or with a file that was unreadable when saving) the check is
        // skipped.
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

                    // Reads the first row as the header
                    if let Some(header_row) = rows_iter.next() {
                        self.csv_headers = header_row.iter().map(cell_to_string).collect();
                    }

                    // Reads the following rows
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

            // Detect the separator (`,` or `;`) from the first line only, then
            // rewind the cursor to the start so the reader reads everything.
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

    /// Test project that points to a temporary file.
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

    /// Writes a temporary CSV and returns its path.
    fn csv_temporaneo(nome: &str, contenuto: &[u8]) -> String {
        let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".to_string());
        let p = std::path::Path::new(&dir).join(nome);
        std::fs::write(&p, contenuto).unwrap();
        p.to_string_lossy().into_owned()
    }

    /// With the right CRC the data is loaded.
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

    /// If the file changes after saving, loading is blocked.
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

        // The file is replaced with different content.
        let nuovo = b"Data,Ora,S1\n2024-01-01,10:00,99.9\n";
        std::fs::write(&percorso, nuovo).unwrap();
        vs.load_data_file();

        let err = vs.csv_load_error.expect("doveva rifiutare il file modificato");
        assert!(
            err.contains("CRC"),
            "il messaggio deve parlare di CRC: {err}"
        );
        // No old or new data must have been loaded.
        assert!(vs.csv_headers.is_empty());
        assert!(vs.csv_rows.is_empty());
        let _ = std::fs::remove_file(&percorso);
    }

    /// The dummy data archive shipped with the project must load in full:
    /// 25 hourly records, 4 sensors, all values between 2 and 8.
    ///
    /// It also checks the boundary time `24:00:00` (the midnight ending the
    /// day, valid in ISO 8601): if the parser rejected it, the last row would
    /// silently disappear from the chart and the 3D map.
    #[test]
    fn archivio_dati_fasulli_si_carica_per_intero() {
        let percorso = "data/example_data.csv";
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

        // The 24:00:00 row is the midnight ending the day: it must fall 24 hours
        // after the first one, not overlap with it.
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

    /// A file separated by `;` with a decimal comma is read correctly: the
    /// separator is detected from the first line, and the values stay numeric.
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

        // The decimal comma must produce real numbers, not split cells.
        let series = crate::data::parse_series(&vs.csv_headers, &vs.csv_rows);
        assert_eq!(series.len(), 2);
        assert_eq!(series[0].name, "Sensore_1");
        assert_eq!(series[0].points.len(), 2);
        assert_eq!(series[0].points[0][1], 21.5, "virgola decimale non convertita");
        assert_eq!(series[1].points[1][1], 21.0);

        let _ = std::fs::remove_file(&percorso);
    }

    /// A project without a recorded CRC still loads (backward compatibility).
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

    /// Saves the project: the file's CRC ends up in the database.
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

        // The project re-read from the database loads the data without errors.
        let mut vs2 = ViewState::new();
        vs2.lang = crate::i18n::Language::It;
        vs2.active_project = Some(progetti[0].clone());
        vs2.load_data_file();
        assert_eq!(vs2.csv_load_error, None, "errore: {:?}", vs2.csv_load_error);

        let _ = std::fs::remove_file(&percorso);
        let _ = std::fs::remove_file(&db_path);
    }
}
