//! Analisi dei dati dei sensori, condivisa fra la vista grafico e l'esportazione PDF.

use chrono::NaiveDateTime;

/// Una serie temporale di un sensore.
#[derive(Clone)]
pub struct Series {
    pub name: String,
    /// Punti `[timestamp unix, temperatura]`.
    pub points: Vec<[f64; 2]>,
    /// Etichette `(data, ora)` parallele a `points`, usate per tooltip e report.
    pub stamps: Vec<(String, String)>,
}

/// Estremi del grafico.
#[derive(Clone, Copy)]
pub struct Bounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

/// Formati di data/ora accettati nei file importati.
const DATETIME_FORMATS: [&str; 8] = [
    "%Y-%m-%d %H:%M:%S",
    "%d/%m/%Y %H:%M:%S",
    "%Y-%m-%d %H:%M",
    "%d/%m/%Y %H:%M",
    "%Y/%m/%d %H:%M:%S",
    "%d-%m-%Y %H:%M:%S",
    "%Y-%m-%d %H:%M:%S%.f",
    "%d/%m/%Y %H:%M:%S%.f",
];

const DATE_FORMATS: [&str; 4] = ["%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y", "%Y/%m/%d"];

/// Converte data + ora in timestamp Unix. Accetta anche celle che contengono già
/// data e ora insieme, oltre ai formati con la virgola decimale.
pub fn parse_timestamp(date_str: &str, time_str: &str) -> Option<f64> {
    let date_str = date_str.trim();
    let time_str = time_str.trim();

    // La cella potrebbe contenere già "data ora" nella prima colonna.
    let mut candidates: Vec<String> = Vec::new();
    if time_str.is_empty() {
        candidates.push(date_str.to_string());
    } else {
        candidates.push(format!("{} {}", date_str, time_str));
    }
    candidates.push(date_str.to_string());

    for c in &candidates {
        for fmt in DATETIME_FORMATS {
            if let Ok(dt) = NaiveDateTime::parse_from_str(c, fmt) {
                return Some(dt.and_utc().timestamp() as f64);
            }
        }
        for fmt in DATE_FORMATS {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(c, fmt) {
                return Some(d.and_hms_opt(0, 0, 0)?.and_utc().timestamp() as f64);
            }
        }
    }
    None
}

/// Converte una cella in numero, tollerando la virgola come separatore decimale.
pub fn parse_number(value: &str) -> Option<f64> {
    let cleaned = value.trim().replace(',', ".");
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<f64>().ok()
}

/// Costruisce le serie a partire dall'intestazione (data, ora, sensori...) e dalle righe.
pub fn parse_series(headers: &[String], rows: &[Vec<String>]) -> Vec<Series> {
    let mut series: Vec<Series> = headers
        .iter()
        .skip(2)
        .map(|name| Series {
            name: name.clone(),
            points: Vec::new(),
            stamps: Vec::new(),
        })
        .collect();

    if series.is_empty() {
        return series;
    }

    for row in rows {
        if row.len() < 2 {
            continue;
        }
        let date_str = row.first().cloned().unwrap_or_default();
        let time_str = row.get(1).cloned().unwrap_or_default();
        let Some(x) = parse_timestamp(&date_str, &time_str) else {
            continue;
        };

        for (i, s) in series.iter_mut().enumerate() {
            if let Some(v) = row.get(i + 2).and_then(|v| parse_number(v)) {
                s.points.push([x, v]);
                s.stamps.push((date_str.clone(), time_str.clone()));
            }
        }
    }

    series
}

/// Calcola gli estremi del grafico su tutte le serie, con un po' di margine verticale.
pub fn bounds(series: &[Series], min_temp: f32, max_temp: f32) -> Option<Bounds> {
    let mut x_min = f64::MAX;
    let mut x_max = f64::MIN;
    let mut y_min = f64::MAX;
    let mut y_max = f64::MIN;
    let mut found = false;

    for s in series {
        for p in &s.points {
            found = true;
            x_min = x_min.min(p[0]);
            x_max = x_max.max(p[0]);
            y_min = y_min.min(p[1]);
            y_max = y_max.max(p[1]);
        }
    }
    if !found {
        return None;
    }

    // Le soglie del progetto entrano sempre nell'area visibile.
    y_min = y_min.min(min_temp as f64);
    y_max = y_max.max(max_temp as f64);

    // Margini di sicurezza.
    let span = (y_max - y_min).max(1.0);
    y_min -= span * 0.08;
    y_max += span * 0.08;
    if x_max - x_min < 1.0 {
        x_min -= 1.0;
        x_max += 1.0;
    }

    Some(Bounds {
        x_min,
        x_max,
        y_min,
        y_max,
    })
}

/// Numero totale di campioni contenuti nelle serie.
pub fn total_points(series: &[Series]) -> usize {
    series.iter().map(|s| s.points.len()).sum()
}

/// Formatta un timestamp Unix come `gg/mm/aaaa hh:mm`.
pub fn format_timestamp(ts: f64) -> String {
    match chrono::DateTime::from_timestamp(ts as i64, 0) {
        Some(dt) => dt.format("%d/%m/%Y %H:%M").to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Converte una riga di celle testuali in celle di testo.
    fn riga(celle: &[&str]) -> Vec<String> {
        celle.iter().map(|s| s.to_string()).collect()
    }

    /// Idem per l'intestazione (le prime due celle non diventano serie).
    fn intestazione(celle: &[&str]) -> Vec<String> {
        riga(celle)
    }

    /// Ogni formato elencato nel README deve essere effettivamente accettato:
    /// se qui fallisce, la documentazione promette qualcosa che il codice non
    /// fa, e va corretta.
    #[test]
    fn tutti_i_formati_documentati_nel_readme() {
        let separati = [
            ("2026-06-01", "08:00:00"), // %Y-%m-%d %H:%M:%S
            ("01/06/2026", "08:00:00"), // %d/%m/%Y %H:%M:%S
            ("2026-06-01", "08:00"),    // %Y-%m-%d %H:%M
            ("01/06/2026", "08:00"),    // %d/%m/%Y %H:%M
            ("2026/06/01", "08:00:00"), // %Y/%m/%d %H:%M:%S
            ("01-06-2026", "08:00:00"), // %d-%m-%Y %H:%M:%S
            ("2026-06-01", "08:00:00.500"),
            ("01/06/2026", "08:00:00.500"),
        ];
        for (d, o) in separati {
            assert!(
                parse_timestamp(d, o).is_some(),
                "formato documentato ma non riconosciuto: {d} + {o}"
            );
        }
    }

    /// Con la seconda colonna vuota, la prima deve accettare la sola data, con
    /// l'ora posta a mezzanotte (come promesso nel README).
    #[test]
    fn sola_data_nella_prima_colonna() {
        for d in ["2026-06-01", "01/06/2026", "01-06-2026", "2026/06/01"] {
            let ts = parse_timestamp(d, "").expect("sola data non riconosciuta");
            let dt = chrono::DateTime::from_timestamp(ts as i64, 0).unwrap();
            assert_eq!(dt.format("%Y-%m-%d").to_string(), "2026-06-01", "data {d}");
            assert_eq!(dt.format("%H:%M").to_string(), "00:00", "data {d}");
        }
    }

    /// La prima colonna può contenere da sola data e ora unite.
    #[test]
    fn data_e_ora_in_una_sola_cella() {
        let ts = parse_timestamp("2026-06-01 08:30:00", "").expect("data+ora non riconosciuta");
        let dt = chrono::DateTime::from_timestamp(ts as i64, 0).unwrap();
        assert_eq!(dt.format("%H:%M").to_string(), "08:30");
    }

    /// La virgola decimale è equipollente al punto, come documentato.
    #[test]
    fn virgola_decimale_equipollente() {
        assert_eq!(parse_number("18.5"), Some(18.5));
        assert_eq!(parse_number("18,5"), Some(18.5));
        assert_eq!(parse_number("  21,0  "), Some(21.0));
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("n/d"), None);
    }

    /// Una riga con data non riconoscibile viene saltata del tutto, mentre una
    /// cella sensore vuota salta solo quel sensore.
    #[test]
    fn righe_e_celle_non_valide_vengono_saltate() {
        let headers = intestazione(&["Data", "Ora", "S1", "S2"]);
        let rows = vec![
            riga(&["2026-06-01", "08:00", "18.5", "19.2"]),
            riga(&["data nonsensica", "08:00", "99.9", "99.9"]),
            riga(&["2026-06-01", "09:00", "", "21.0"]),
        ];
        let series = parse_series(&headers, &rows);

        assert_eq!(series.len(), 2);
        // La riga illeggibile non deve lasciare traccia in nessuna serie.
        assert_eq!(series[0].points.len(), 1, "S1: la riga valida + la cella vuota");
        assert_eq!(series[1].points.len(), 2, "S2: entrambe le righe valide");
        assert_eq!(series[0].points[0][1], 18.5);
        assert_eq!(series[1].points[1][1], 21.0);
    }

    /// Le prime due colonne non diventano serie: i sensori iniziano dalla terza.
    #[test]
    fn le_prime_due_colonne_non_serie_sensori() {
        let headers = intestazione(&["Data", "Ora", "Sensore_1", "Sensore_2"]);
        let rows = vec![riga(&["2026-06-01", "08:00", "18.5", "19.2"])];
        let series = parse_series(&headers, &rows);

        let nomi: Vec<&str> = series.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(nomi, vec!["Sensore_1", "Sensore_2"]);
    }

    /// Una serie con tutte le celle numeriche vuote non produce punti, ma le
    /// altre serie restano intatte.
    #[test]
    fn serie_vuota_non_blocca_le_altre() {
        let headers = intestazione(&["Data", "Ora", "S1", "S2"]);
        let rows = vec![
            riga(&["2026-06-01", "08:00", "", "19.2"]),
            riga(&["2026-06-01", "09:00", "", "21.0"]),
        ];
        let series = parse_series(&headers, &rows);
        assert!(series[0].points.is_empty());
        assert_eq!(series[1].points.len(), 2);
    }
}
