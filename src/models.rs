use crate::i18n::{self, Language};
use rusqlite::Connection;

#[derive(Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub container_type: String,
    pub min_temp: f32,
    pub max_temp: f32,
    pub data_file_path: String,
    pub data_crc32: Option<String>,
}

/// Dimensioni del parallelepipedo che rappresenta il container (in metri).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneDims {
    pub w: f32,
    pub h: f32,
    pub d: f32,
}

impl Default for SceneDims {
    fn default() -> Self {
        Self {
            w: 6.0,
            h: 2.5,
            d: 2.4,
        }
    }
}

impl SceneDims {
    /// Metà diagonale, usata per inquadrare l'oggetto e per i controlli.
    pub fn half_diagonal(self) -> f32 {
        (self.w * self.w + self.h * self.h + self.d * self.d).sqrt() * 0.5
    }

    /// Vincola le dimensioni a un intervallo ragionevole.
    pub fn sanitized(self) -> Self {
        Self {
            w: self.w.clamp(0.5, 60.0),
            h: self.h.clamp(0.5, 60.0),
            d: self.d.clamp(0.5, 60.0),
        }
    }
}

/// Posizione di un sensore su una faccia del parallelepipedo.
///
/// `face` è l'indice della faccia (0..5); `u`, `v` sono coordinate normalizzate
/// `[0, 1]` sulla faccia.
#[derive(Clone, Debug, PartialEq)]
pub struct SensorPos {
    pub name: String,
    pub face: usize,
    pub u: f32,
    pub v: f32,
}

/// Le schede della barra multifunzione (una per vista).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectedTab {
    Management,
    SearchList,
    DataGrid,
    ChartPlot,
    SensorMap,
}

impl SelectedTab {
    pub fn label(&self, lang: Language) -> &'static str {
        i18n::t(
            lang,
            match self {
                SelectedTab::Management => "Gestione",
                SelectedTab::SearchList => "Ricerca",
                SelectedTab::DataGrid => "Dati",
                SelectedTab::ChartPlot => "Grafico",
                SelectedTab::SensorMap => "Sensori 3D",
            },
        )
    }

    /// Descrizione breve mostrata nella barra di stato.
    pub fn description(&self, lang: Language) -> &'static str {
        i18n::t(
            lang,
            match self {
                SelectedTab::Management => "Inserimento e modifica progetti",
                SelectedTab::SearchList => "Elenco e ricerca progetti",
                SelectedTab::DataGrid => "Griglia dati sensori",
                SelectedTab::ChartPlot => "Grafico termico",
                SelectedTab::SensorMap => "Posizione dei sensori nel container 3D",
            },
        )
    }

    pub const ALL: [SelectedTab; 5] = [
        SelectedTab::Management,
        SelectedTab::SearchList,
        SelectedTab::DataGrid,
        SelectedTab::ChartPlot,
        SelectedTab::SensorMap,
    ];
}

/// Comandi emessi dal menu e dalla barra multifunzione.
#[derive(Clone, PartialEq)]
pub enum Command {
    NewProject,
    SaveProject,
    DeleteProject,
    ReloadProjects,
    ReloadData,
    PickDataFile,
    SwitchTab(SelectedTab),
    ToggleTheme,
    OpenSettings,
    ExportPdf,
    ExportMapPdf,
    ResetMapView,
    ClearSearch,
    TogglePoints,
    ToggleLegend,
    ResetZoom,
    About,
    Quit,
}

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn new() -> Self {
        Self::open_at("advanced_projects.db")
    }

    /// Apre (o crea) il database al percorso indicato.
    pub fn open_at(percorso: &str) -> Self {
        let conn = Connection::open(percorso)
            .expect("Impossibile aprire il database SQLite");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                container_type TEXT NOT NULL,
                min_temp REAL NOT NULL,
                max_temp REAL NOT NULL,
                data_file_path TEXT NOT NULL
            )",
            [],
        )
        .expect("Errore creazione tabella projects");

        // Migrazione: aggiunge la colonna per il CRC32 del file dati, se il
        // database è stato creato da una versione precedente. Si interroga lo
        // schema invece di ignorare l'errore, così un errore vero (es. database
        // in sola lettura) non viene silenziosamente nascosto.
        let ha_crc32 = conn
            .prepare("PRAGMA table_info(projects)")
            .and_then(|mut stmt| {
                let righe = stmt.query_map([], |r| r.get::<_, String>(1))?; // nome colonna
                for nome in righe {
                    if nome? == "data_crc32" {
                        return Ok(true);
                    }
                }
                Ok(false)
            })
            .expect("Impossibile leggere lo schema della tabella projects");
        if !ha_crc32 {
            conn.execute("ALTER TABLE projects ADD COLUMN data_crc32 TEXT", [])
                .expect("Impossibile aggiungere la colonna data_crc32");
        }

        // Dimensioni del parallelepipedo usato nella mappa 3D (una per progetto).
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sensor_layouts (
                project_id INTEGER PRIMARY KEY,
                width REAL NOT NULL,
                height REAL NOT NULL,
                depth REAL NOT NULL
            )",
            [],
        )
        .expect("Errore creazione tabella sensor_layouts");

        // Posizione dei sensori sul parallelepipedo. La chiave primaria composta
        // (project_id, sensor_name) impedisce di posizionare due volte lo stesso
        // sensore dello stesso progetto.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sensor_positions (
                project_id INTEGER NOT NULL,
                sensor_name TEXT NOT NULL,
                face INTEGER NOT NULL,
                u REAL NOT NULL,
                v REAL NOT NULL,
                PRIMARY KEY (project_id, sensor_name)
            )",
            [],
        )
        .expect("Errore creazione tabella sensor_positions");

        Self { conn }
    }

    pub fn load_all_projects(&self) -> Vec<Project> {
        let mut projects = Vec::new();
        let mut stmt = match self.conn.prepare("SELECT id, name, container_type, min_temp, max_temp, data_file_path, data_crc32 FROM projects ORDER BY id ASC") {
            Ok(s) => s,
            Err(_) => return projects,
        };

        let iter = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                container_type: row.get(2)?,
                min_temp: row.get(3)?,
                max_temp: row.get(4)?,
                data_file_path: row.get(5)?,
                data_crc32: row.get(6).ok(),
            })
        });

        if let Ok(project_iter) = iter {
            for p in project_iter {
                if let Ok(proj) = p {
                    projects.push(proj);
                }
            }
        }
        projects
    }

    // -----------------------------------------------------------------------
    // Mappa 3D: dimensioni del container
    // -----------------------------------------------------------------------

    /// Legge le dimensioni salvate del parallelepipedo per un progetto.
    pub fn load_scene_dims(&self, project_id: i64) -> Option<SceneDims> {
        self.conn
            .query_row(
                "SELECT width, height, depth FROM sensor_layouts WHERE project_id = ?1",
                [project_id],
                |row| {
                    Ok(SceneDims {
                        w: row.get(0)?,
                        h: row.get(1)?,
                        d: row.get(2)?,
                    })
                },
            )
            .ok()
    }

    /// Salva (crea o aggiorna) le dimensioni del parallelepipedo per un progetto.
    pub fn save_scene_dims(&self, project_id: i64, dims: SceneDims) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO sensor_layouts (project_id, width, height, depth)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(project_id) DO UPDATE SET
                     width = excluded.width,
                     height = excluded.height,
                     depth = excluded.depth",
                (project_id, dims.w, dims.h, dims.d),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    // -----------------------------------------------------------------------
    // Mappa 3D: posizione dei sensori
    // -----------------------------------------------------------------------

    /// Carica le posizioni dei sensori salvate per un progetto.
    pub fn load_sensor_positions(&self, project_id: i64) -> Vec<SensorPos> {
        let mut out = Vec::new();
        let mut stmt = match self.conn.prepare(
            "SELECT sensor_name, face, u, v FROM sensor_positions
             WHERE project_id = ?1 ORDER BY sensor_name ASC",
        ) {
            Ok(s) => s,
            Err(_) => return out,
        };

        let iter = stmt.query_map([project_id], |row| {
            Ok(SensorPos {
                name: row.get(0)?,
                face: row.get::<_, i64>(1)? as usize,
                u: row.get(2)?,
                v: row.get(3)?,
            })
        });

        if let Ok(rows) = iter {
            for r in rows.flatten() {
                out.push(r);
            }
        }
        out
    }

    /// Inserisce la posizione di un sensore. Restituisce `Err` se il sensore è
    /// già presente per lo stesso progetto (vincolo di unicità del database).
    pub fn insert_sensor_position(
        &self,
        project_id: i64,
        pos: &SensorPos,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO sensor_positions (project_id, sensor_name, face, u, v)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                (project_id, &pos.name, pos.face as i64, pos.u, pos.v),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Elimina la posizione di un sensore.
    pub fn delete_sensor_position(&self, project_id: i64, name: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM sensor_positions WHERE project_id = ?1 AND sensor_name = ?2",
                (project_id, name),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Aggiorna la posizione di un sensore già inserito (esito di un
    /// trascinamento sulla vista 3D). Restituisce `Err` se il sensore non
    /// era presente: non è una creazione, quindi una riga mancante è un errore.
    pub fn update_sensor_position(
        &self,
        project_id: i64,
        pos: &SensorPos,
    ) -> Result<(), String> {
        let righe = self
            .conn
            .execute(
                "UPDATE sensor_positions SET face = ?3, u = ?4, v = ?5
                 WHERE project_id = ?1 AND sensor_name = ?2",
                (project_id, &pos.name, pos.face as i64, pos.u, pos.v),
            )
            .map_err(|e| e.to_string())?;
        if righe == 0 {
            return Err(format!("posizione del sensore {} assente", pos.name));
        }
        Ok(())
    }

    /// Elimina tutte le posizioni dei sensori di un progetto.
    pub fn delete_all_sensor_positions(&self, project_id: i64) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM sensor_positions WHERE project_id = ?1",
                [project_id],
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Percorso temporaneo unico per un database di prova.
    fn db_temporaneo(nome: &str) -> String {
        let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".to_string());
        let p = std::path::Path::new(&dir).join(nome);
        let _ = std::fs::remove_file(&p);
        p.to_string_lossy().into_owned()
    }

    /// Un database creato dalla versione precedente (senza `data_crc32`) viene
    /// migrato: la colonna viene aggiunta e le righe vecchie restano leggibili
    /// con CRC assente, quindi la verifica al caricamento viene saltata.
    #[test]
    fn migra_un_database_della_vecchia_versione() {
        let percorso = db_temporaneo("thermite_migrazione_prova.db");

        // Schema vecchio, esattamente quello delle versioni precedenti.
        {
            let conn = Connection::open(&percorso).unwrap();
            conn.execute(
                "CREATE TABLE projects (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    container_type TEXT NOT NULL,
                    min_temp REAL NOT NULL,
                    max_temp REAL NOT NULL,
                    data_file_path TEXT NOT NULL
                )",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO projects (name, container_type, min_temp, max_temp, data_file_path)
                 VALUES ('Vecchio', 'Container 20piedi', 0.0, 30.0, 'data/vecchio.csv')",
                [],
            )
            .unwrap();
        }

        // L'apertura con la versione nuova deve migrare lo schema.
        let db = Database::open_at(&percorso);
        let progetti = db.load_all_projects();
        assert_eq!(progetti.len(), 1, "la riga vecchia deve sopravvivere");
        assert_eq!(progetti[0].name, "Vecchio");
        assert_eq!(
            progetti[0].data_crc32, None,
            "le righe vecchie non hanno CRC: la verifica viene saltata"
        );

        // Riaprire non deve fallire né duplicare la colonna (ALTER già applicato).
        let db2 = Database::open_at(&percorso);
        assert_eq!(db2.load_all_projects().len(), 1);

        let _ = std::fs::remove_file(&percorso);
    }

    /// Su un database nuovo la colonna è presente e il CRC scritto al
    /// salvataggio viene riletto correttamente.
    #[test]
    fn database_nuovo_ha_la_colonna_crc() {
        let percorso = db_temporaneo("thermite_nuovo_prova.db");
        let db = Database::open_at(&percorso);

        db.conn
            .execute(
                "INSERT INTO projects (name, container_type, min_temp, max_temp, data_file_path, data_crc32)
                 VALUES ('Nuovo', 'Container 40piedi', 0.0, 30.0, 'data/nuovo.csv', 'CBF43926')",
                [],
            )
            .unwrap();

        let progetti = db.load_all_projects();
        assert_eq!(progetti[0].data_crc32.as_deref(), Some("CBF43926"));

        let _ = std::fs::remove_file(&percorso);
    }
}
