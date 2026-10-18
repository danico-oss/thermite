use crate::i18n::{self, Language};
use crate::scene3d::Vec3;
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

/// Dimensions of the parallelepiped that represents the container (in meters).
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
    /// Half diagonal, used to frame the object and for the controls.
    pub fn half_diagonal(self) -> f32 {
        (self.w * self.w + self.h * self.h + self.d * self.d).sqrt() * 0.5
    }

    /// Constrains the dimensions to a reasonable range.
    pub fn sanitized(self) -> Self {
        Self {
            w: self.w.clamp(0.5, 60.0),
            h: self.h.clamp(0.5, 60.0),
            d: self.d.clamp(0.5, 60.0),
        }
    }
}

/// Position of a sensor on (or near) the parallelepiped.
///
/// `face` is the face index (0..5); `u`, `v` are normalized coordinates
/// `[0, 1]` on the face: together they describe the point **on the wall** where
/// the sensor was placed with the click.
///
/// `pos` takes precedence when present and is the actual position in
/// **normalized** coordinates, that is, fractions of the half dimension of
/// the container (`x / (width/2)`, etc.). It is set as soon as the
/// sensor is dragged; from that moment on, the point no longer necessarily
/// lies on any wall. Normalizing it makes a dragged sensor follow the
/// container when its dimensions change, exactly like the ones that stayed
/// on the wall.
///
/// `Vec3` is defined in the geometry module: depending on it from here
/// (rather than keeping a tuple) avoids conversions at every point where the
/// position is drawn or projected.
#[derive(Clone, Debug, PartialEq)]
pub struct SensorPos {
    pub name: String,
    pub face: usize,
    pub u: f32,
    pub v: f32,
    pub pos: Option<Vec3>,
}

/// The ribbon tabs (one per view).
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

    /// Short description shown in the status bar.
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

/// Commands emitted by the menu and the ribbon.
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

    /// Opens (or creates) the database at the given path.
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

        // Migration: adds the column for the CRC32 of the data file, if the
        // database was created by a previous version. The schema is queried
        // instead of ignoring the error, so that a real error (e.g. a read-only
        // database) is not silently hidden.
        let ha_crc32 = conn
            .prepare("PRAGMA table_info(projects)")
            .and_then(|mut stmt| {
                let righe = stmt.query_map([], |r| r.get::<_, String>(1))?; // column name
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

        // Dimensions of the parallelepiped used in the 3D map (one per project).
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

        // Sensor positions on the parallelepiped. The composite primary key
        // (project_id, sensor_name) prevents placing the same sensor of the
        // same project twice.
        //
        // `face`, `u`, `v` describe the point on the wall at which the sensor
        // was placed; `pos_x`, `pos_y`, `pos_z` (optional) are the normalized
        // actual position, written when the sensor is dragged and can therefore
        // detach from the wall.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sensor_positions (
                project_id INTEGER NOT NULL,
                sensor_name TEXT NOT NULL,
                face INTEGER NOT NULL,
                u REAL NOT NULL,
                v REAL NOT NULL,
                pos_x REAL,
                pos_y REAL,
                pos_z REAL,
                PRIMARY KEY (project_id, sensor_name)
            )",
            [],
        )
        .expect("Errore creazione tabella sensor_positions");

        // Migration: adds the actual-position columns to databases created by
        // a previous version. As with `data_crc32`, the schema is queried
        // instead of ignoring the error, so that a real error is not silently
        // hidden.
        let colonne_sensori = conn
            .prepare("PRAGMA table_info(sensor_positions)")
            .and_then(|mut stmt| {
                let righe = stmt.query_map([], |r| r.get::<_, String>(1))?; // column name
                let mut nomi = Vec::new();
                for nome in righe {
                    nomi.push(nome?);
                }
                Ok(nomi)
            })
            .expect("Impossibile leggere lo schema della tabella sensor_positions");
        for (colonna, ddl) in [
            ("pos_x", "ALTER TABLE sensor_positions ADD COLUMN pos_x REAL"),
            ("pos_y", "ALTER TABLE sensor_positions ADD COLUMN pos_y REAL"),
            ("pos_z", "ALTER TABLE sensor_positions ADD COLUMN pos_z REAL"),
        ] {
            if !colonne_sensori.iter().any(|c| c == colonna) {
                conn.execute(ddl, [])
                    .expect("Impossibile aggiungere la colonna di posizione");
            }
        }

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
    // 3D map: container dimensions
    // -----------------------------------------------------------------------

    /// Reads the saved dimensions of the parallelepiped for a project.
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

    /// Saves (creates or updates) the dimensions of the parallelepiped for a project.
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
    // 3D map: sensor positions
    // -----------------------------------------------------------------------

    /// Loads the sensor positions saved for a project.
    pub fn load_sensor_positions(&self, project_id: i64) -> Vec<SensorPos> {
        let mut out = Vec::new();
        let mut stmt = match self.conn.prepare(
            "SELECT sensor_name, face, u, v, pos_x, pos_y, pos_z FROM sensor_positions
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
                pos: leggi_posizione(row.get(4)?, row.get(5)?, row.get(6)?),
            })
        });

        if let Ok(rows) = iter {
            for r in rows.flatten() {
                out.push(r);
            }
        }
        out
    }

    /// Inserts the position of a sensor. Returns `Err` if the sensor is
    /// already present for the same project (database uniqueness constraint).
    pub fn insert_sensor_position(
        &self,
        project_id: i64,
        pos: &SensorPos,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO sensor_positions
                 (project_id, sensor_name, face, u, v, pos_x, pos_y, pos_z)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                (
                    project_id,
                    &pos.name,
                    pos.face as i64,
                    pos.u,
                    pos.v,
                    pos.pos.map(|p| p.x),
                    pos.pos.map(|p| p.y),
                    pos.pos.map(|p| p.z),
                ),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Deletes the position of a sensor.
    pub fn delete_sensor_position(&self, project_id: i64, name: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM sensor_positions WHERE project_id = ?1 AND sensor_name = ?2",
                (project_id, name),
            )
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Updates the position of an already inserted sensor (the outcome of a
    /// drag on the 3D view). Returns `Err` if the sensor was not present:
    /// this is not a creation, so a missing row is an error.
    pub fn update_sensor_position(
        &self,
        project_id: i64,
        pos: &SensorPos,
    ) -> Result<(), String> {
        let righe = self
            .conn
            .execute(
                "UPDATE sensor_positions
                 SET face = ?3, u = ?4, v = ?5, pos_x = ?6, pos_y = ?7, pos_z = ?8
                 WHERE project_id = ?1 AND sensor_name = ?2",
                (
                    project_id,
                    &pos.name,
                    pos.face as i64,
                    pos.u,
                    pos.v,
                    pos.pos.map(|p| p.x),
                    pos.pos.map(|p| p.y),
                    pos.pos.map(|p| p.z),
                ),
            )
            .map_err(|e| e.to_string())?;
        if righe == 0 {
            return Err(format!("posizione del sensore {} assente", pos.name));
        }
        Ok(())
    }

    /// Deletes all the sensor positions of a project.
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

/// Reconstructs the actual position from the three database columns.
///
/// If even only one of the three is missing (a row written by a previous
/// version, which did not have them) the position is absent: the sensor then
/// stays glued to the `face` wall at the `u`, `v` coordinates.
fn leggi_posizione(x: Option<f32>, y: Option<f32>, z: Option<f32>) -> Option<Vec3> {
    match (x, y, z) {
        (Some(x), Some(y), Some(z)) => Some(Vec3::new(x, y, z)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unique temporary path for a test database.
    fn db_temporaneo(nome: &str) -> String {
        let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".to_string());
        let p = std::path::Path::new(&dir).join(nome);
        let _ = std::fs::remove_file(&p);
        p.to_string_lossy().into_owned()
    }

    /// A database created by the previous version (without `data_crc32`) is
    /// migrated: the column is added and the old rows remain readable with a
    /// missing CRC, so the check at load time is skipped.
    #[test]
    fn migra_un_database_della_vecchia_versione() {
        let percorso = db_temporaneo("thermite_migrazione_prova.db");

        // Old schema, exactly the one of the previous versions.
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

        // Opening with the new version must migrate the schema.
        let db = Database::open_at(&percorso);
        let progetti = db.load_all_projects();
        assert_eq!(progetti.len(), 1, "la riga vecchia deve sopravvivere");
        assert_eq!(progetti[0].name, "Vecchio");
        assert_eq!(
            progetti[0].data_crc32, None,
            "le righe vecchie non hanno CRC: la verifica viene saltata"
        );

        // Reopening must not fail nor duplicate the column (ALTER already applied).
        let db2 = Database::open_at(&percorso);
        assert_eq!(db2.load_all_projects().len(), 1);

        let _ = std::fs::remove_file(&percorso);
    }

    /// On a new database the column is present and the CRC written at
    /// save time is read back correctly.
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

    /// An actual position (written when the sensor is dragged) is saved and
    /// read back identical; the point at which it had been placed remains
    /// preserved in `face`, `u`, `v`.
    #[test]
    fn la_posizione_trascinata_passa_attraverso_il_database() {
        let percorso = db_temporaneo("thermite_posizione_libera_prova.db");
        let db = Database::open_at(&percorso);

        // Sensor still glued to the wall: no actual position.
        let sulla_parete = SensorPos {
            name: "S1".to_string(),
            face: 4,
            u: 0.5,
            v: 0.25,
            pos: None,
        };
        db.insert_sensor_position(1, &sulla_parete).unwrap();
        assert_eq!(
            db.load_sensor_positions(1),
            vec![sulla_parete.clone()],
            "un sensore appena posizionato non ha posizione effettiva"
        );

        // Dragged: the actual position takes precedence and is read back
        // exactly as it is, with face, u, v intact.
        let trascinato = SensorPos {
            pos: Some(Vec3::new(1.75, -0.5, 0.2)),
            ..sulla_parete.clone()
        };
        db.update_sensor_position(1, &trascinato).unwrap();
        assert_eq!(
            db.load_sensor_positions(1),
            vec![trascinato.clone()],
            "la posizione trascinata deve tornare identica"
        );

        // Moving the sensor again rewrites the position, it does not accumulate.
        let spostato = SensorPos {
            pos: Some(Vec3::new(-2.0, 0.0, 1.0)),
            ..trascinato.clone()
        };
        db.update_sensor_position(1, &spostato).unwrap();
        assert_eq!(db.load_sensor_positions(1), vec![spostato]);

        // A sensor never dragged remains described by the wall alone.
        let altro = SensorPos {
            name: "S2".to_string(),
            face: 1,
            u: 0.1,
            v: 0.9,
            pos: None,
        };
        db.insert_sensor_position(1, &altro).unwrap();
        let caricate = db.load_sensor_positions(1);
        assert_eq!(caricate.len(), 2);
        assert_eq!(caricate[1], altro);

        let _ = std::fs::remove_file(&percorso);
    }

    /// A database created before the sensors could be dragged does not have
    /// the actual-position columns: opening adds them and the old rows are
    /// read as sensors that stayed on the wall.
    #[test]
    fn migra_la_tabella_sensori_del_database_vecchio() {
        let percorso = db_temporaneo("thermite_migrazione_sensori_prova.db");

        // Old schema, exactly the one of the previous versions.
        {
            let conn = Connection::open(&percorso).unwrap();
            conn.execute(
                "CREATE TABLE sensor_positions (
                    project_id INTEGER NOT NULL,
                    sensor_name TEXT NOT NULL,
                    face INTEGER NOT NULL,
                    u REAL NOT NULL,
                    v REAL NOT NULL,
                    PRIMARY KEY (project_id, sensor_name)
                )",
                [],
            )
            .unwrap();
            conn.execute(
                "INSERT INTO sensor_positions (project_id, sensor_name, face, u, v)
                 VALUES (1, 'S1', 4, 0.5, 0.25)",
                [],
            )
            .unwrap();
        }

        // Opening with the new version must migrate the schema.
        let db = Database::open_at(&percorso);
        let caricate = db.load_sensor_positions(1);
        assert_eq!(caricate.len(), 1, "la riga vecchia deve sopravvivere");
        assert_eq!(caricate[0].name, "S1");
        assert_eq!(
            caricate[0].pos, None,
            "una riga senza posizione è un sensore ancora sulla parete"
        );

        // A second opening (ALTER already applied) must be able to update the
        // row, that is, write on the just-added columns.
        let db2 = Database::open_at(&percorso);
        let spostato = SensorPos {
            pos: Some(Vec3::new(1.0, 1.0, 1.0)),
            ..caricate[0].clone()
        };
        db2.update_sensor_position(1, &spostato).unwrap();
        assert_eq!(db2.load_sensor_positions(1), vec![spostato]);

        let _ = std::fs::remove_file(&percorso);
    }
}
