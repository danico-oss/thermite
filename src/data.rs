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
