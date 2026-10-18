//! Sensor data parsing, shared between the chart view and the PDF export.

use chrono::NaiveDateTime;

/// A time series of a sensor.
#[derive(Clone)]
pub struct Series {
    pub name: String,
    /// Points `[unix timestamp, temperature]`.
    pub points: Vec<[f64; 2]>,
    /// `(date, time)` labels parallel to `points`, used for tooltips and reports.
    pub stamps: Vec<(String, String)>,
}

/// Chart bounds.
#[derive(Clone, Copy)]
pub struct Bounds {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
}

/// Date/time formats accepted in imported files.
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

/// Converts date + time into a Unix timestamp. It also accepts cells that already
/// contain date and time together, plus the formats with the decimal comma.
pub fn parse_timestamp(date_str: &str, time_str: &str) -> Option<f64> {
    let date_str = date_str.trim();
    let time_str = time_str.trim();

    // Hour 24 denotes the midnight at the end of the day in ISO 8601, but the
    // chrono library accepts only 00-23 and rejects the text as a whole. Without
    // this conversion the row would fall back to the "date only" case, ending
    // up overlapped on the midnight at the start of the day: the sample would
    // not be lost, but it would jump 24 hours to the left on chart and map.
    // The conversion applies both to the time column and to a single cell that
    // already contains "date time", as documented in the README.
    let (date_norm, ore_date) = normalizza_ora_24(date_str);
    let (time_norm, ore_time) = normalizza_ora_24(time_str);
    // In a malformed file both cells could carry "24": it is taken
    // only once, never doubled.
    let ore_scarto = ore_date.max(ore_time);
    let date_norm = date_norm.as_str();
    let time_norm = time_norm.as_str();

    // The cell could already contain "date time" in the first column.
    let mut candidates: Vec<String> = Vec::new();
    if time_norm.is_empty() {
        candidates.push(date_norm.to_string());
    } else {
        candidates.push(format!("{} {}", date_norm, time_norm));
    }
    candidates.push(date_norm.to_string());

    for c in &candidates {
        for fmt in DATETIME_FORMATS {
            if let Ok(dt) = NaiveDateTime::parse_from_str(c, fmt) {
                return Some(dt.and_utc().timestamp() as f64 + ore_scarto);
            }
        }
        for fmt in DATE_FORMATS {
            if let Ok(d) = chrono::NaiveDate::parse_from_str(c, fmt) {
                return Some(d.and_hms_opt(0, 0, 0)?.and_utc().timestamp() as f64 + ore_scarto);
            }
        }
    }
    None
}

/// Rewrites a time `24:MM[:SS]` as `00:MM[:SS]` and returns the seconds to
/// add to the result (86400, that is 24 hours), or the unchanged text and `0.0`.
///
/// The "24" must be recognized only when it is at the head of the text or
/// follows a space, so a date like `2024:...` or a sensor name is left alone.
fn normalizza_ora_24(testo: &str) -> (String, f64) {
    for (i, _) in testo.match_indices("24:") {
        let in_testa = i == 0;
        let dopo_spazio = i > 0 && testo.as_bytes()[i - 1] == b' ';
        if !(in_testa || dopo_spazio) {
            continue;
        }
        let resto = &testo[i + 3..];
        // There must be at least one minute digit, otherwise it is not a time.
        if !resto.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        let mut out = String::with_capacity(testo.len());
        out.push_str(&testo[..i]);
        out.push_str("00:");
        out.push_str(resto);
        return (out, 24.0 * 3600.0);
    }
    (testo.to_string(), 0.0)
}

/// Converts a cell into a number, tolerating the comma as decimal separator.
pub fn parse_number(value: &str) -> Option<f64> {
    let cleaned = value.trim().replace(',', ".");
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<f64>().ok()
}

/// Detects the field separator of a CSV file by looking at its first row.
///
/// Counts the occurrences of `,` and `;` that are **outside** the quotes and
/// returns the most frequent one: this way a European file separated by `;` with
/// a decimal comma (`01/06/2026;08:00;18,5`) is read correctly.
///
/// On a tie the comma wins, as it is the historical separator of the
/// application. If the row is empty it still returns the
/// comma, the default behavior of the `csv` crate.
///
/// The analysis only looks at header rows: a data row is never
/// examined, so a text field containing `;` cannot fool the
/// detection.
pub fn detect_separator(prima_riga: &str) -> u8 {
    let bytes = prima_riga.as_bytes();
    let mut virgole = 0usize;
    let mut punti_e_virgola = 0usize;
    let mut dentro_virgolette = false;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            // End of row: the separator cannot lie beyond it.
            b'\n' | b'\r' => break,
            b'"' => {
                // A pair of quotes inside a field is an escape (`""`),
                // not a closing quote.
                if dentro_virgolette && i + 1 < bytes.len() && bytes[i + 1] == b'"' {
                    i += 2;
                    continue;
                }
                dentro_virgolette = !dentro_virgolette;
            }
            b',' if !dentro_virgolette => virgole += 1,
            b';' if !dentro_virgolette => punti_e_virgola += 1,
            _ => {}
        }
        i += 1;
    }

    if punti_e_virgola > virgole { b';' } else { b',' }
}

/// Builds the series from the header (date, time, sensors...) and from the rows.
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

/// Computes the chart bounds over all the series, with a bit of vertical margin.
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

    // The project thresholds always fall within the visible area.
    y_min = y_min.min(min_temp as f64);
    y_max = y_max.max(max_temp as f64);

    // Safety margins.
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

/// Total number of samples contained in the series.
pub fn total_points(series: &[Series]) -> usize {
    series.iter().map(|s| s.points.len()).sum()
}

/// Formats a Unix timestamp as `dd/mm/yyyy hh:mm`.
pub fn format_timestamp(ts: f64) -> String {
    match chrono::DateTime::from_timestamp(ts as i64, 0) {
        Some(dt) => dt.format("%d/%m/%Y %H:%M").to_string(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Converts a row of string cells into text cells.
    fn riga(celle: &[&str]) -> Vec<String> {
        celle.iter().map(|s| s.to_string()).collect()
    }

    /// Same for the header (the first two cells do not become series).
    fn intestazione(celle: &[&str]) -> Vec<String> {
        riga(celle)
    }

    /// Every format listed in the README must actually be accepted:
    /// if this fails, the documentation promises something the code does not
    /// do, and it must be fixed.
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

    /// With the second column empty, the first must accept the date alone, with
    /// the time set to midnight (as promised in the README).
    #[test]
    fn sola_data_nella_prima_colonna() {
        for d in ["2026-06-01", "01/06/2026", "01-06-2026", "2026/06/01"] {
            let ts = parse_timestamp(d, "").expect("sola data non riconosciuta");
            let dt = chrono::DateTime::from_timestamp(ts as i64, 0).unwrap();
            assert_eq!(dt.format("%Y-%m-%d").to_string(), "2026-06-01", "data {d}");
            assert_eq!(dt.format("%H:%M").to_string(), "00:00", "data {d}");
        }
    }

    /// The first column can contain date and time together on its own.
    #[test]
    fn data_e_ora_in_una_sola_cella() {
        let ts = parse_timestamp("2026-06-01 08:30:00", "").expect("data+ora non riconosciuta");
        let dt = chrono::DateTime::from_timestamp(ts as i64, 0).unwrap();
        assert_eq!(dt.format("%H:%M").to_string(), "08:30");
    }

    /// The decimal comma is equivalent to the dot, as documented.
    #[test]
    fn virgola_decimale_equipollente() {
        assert_eq!(parse_number("18.5"), Some(18.5));
        assert_eq!(parse_number("18,5"), Some(18.5));
        assert_eq!(parse_number("  21,0  "), Some(21.0));
        assert_eq!(parse_number(""), None);
        assert_eq!(parse_number("n/d"), None);
    }

    /// A row with an unrecognizable date is skipped entirely, while an
    /// empty sensor cell skips only that sensor.
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
        // The unreadable row must not leave a trace in any series.
        assert_eq!(series[0].points.len(), 1, "S1: la riga valida + la cella vuota");
        assert_eq!(series[1].points.len(), 2, "S2: entrambe le righe valide");
        assert_eq!(series[0].points[0][1], 18.5);
        assert_eq!(series[1].points[1][1], 21.0);
    }

    /// The first two columns do not become series: sensors start from the third.
    #[test]
    fn le_prime_due_colonne_non_serie_sensori() {
        let headers = intestazione(&["Data", "Ora", "Sensore_1", "Sensore_2"]);
        let rows = vec![riga(&["2026-06-01", "08:00", "18.5", "19.2"])];
        let series = parse_series(&headers, &rows);

        let nomi: Vec<&str> = series.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(nomi, vec!["Sensore_1", "Sensore_2"]);
    }

    /// A series with all the numeric cells empty produces no points, but the
    /// other series remain untouched.
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

    // --- Field separator detection ---

    #[test]
    fn separatore_virgola_quando_usata() {
        assert_eq!(detect_separator("Data,Ora,Sensore_1,Sensore_2"), b',');
        assert_eq!(detect_separator("Data,Ora"), b',');
    }

    #[test]
    fn separatore_punto_e_virgola_quando_usato() {
        assert_eq!(detect_separator("Data;Ora;Sensore_1;Sensore_2"), b';');
        assert_eq!(detect_separator("Data;Ora"), b';');
    }

    /// On equal counts the comma wins, the historical separator.
    #[test]
    fn separatore_pari_conteggi_vince_la_virgola() {
        assert_eq!(detect_separator("a,b;c"), b',');
    }

    /// An empty row (or one with only a newline) must not choose at random.
    #[test]
    fn separatore_riga_vuota() {
        assert_eq!(detect_separator(""), b',');
        assert_eq!(detect_separator("\n"), b',');
        assert_eq!(detect_separator("\r\n"), b',');
    }

    /// Separators enclosed in quotes belong to the field, they are not
    /// separators: without this check a comma-separated file with a
    /// field containing a `;` would be mistaken for a European file.
    #[test]
    fn separatore_ignora_il_contenuto_delle_virgolette() {
        // A `;` inside a field must not prevail: the separator is the comma.
        assert_eq!(detect_separator("\"Nota;urgente\",Ora,S1"), b',');
        // Likewise a comma inside a field of a file separated by `;`.
        assert_eq!(detect_separator("\"Nota,urgente\";Ora;S1"), b';');
        // An empty field enclosed in quotes changes nothing.
        assert_eq!(detect_separator("\"\",Ora,S1"), b',');
    }

    /// Double quotes inside a field are an escape, not a closing quote.
    #[test]
    fn separatore_virgolette_escape() {
        assert_eq!(detect_separator("\"dici \"\"sì\"\"\",Ora,S1"), b',');
    }

    /// The first row goes past the line terminator: a field further to the right
    /// than the head must not be counted twice or ignored.
    #[test]
    fn separatore_si_ferma_alla_fine_riga() {
        assert_eq!(detect_separator("Data;Ora;S1\n2026-06-01,08:00,18.5"), b';');
    }

    /// Realistic bounds: European header with a decimal comma in the data.
    #[test]
    fn separatore_file_europeo_completo() {
        let prima_riga = "Data;Ora;Sensore 1;Sensore 2";
        assert_eq!(detect_separator(prima_riga), b';');
    }

    // --- End-of-day time ---

    /// `24:00:00` is the midnight that closes the day in ISO 8601. It must
    /// be worth 24 hours after the midnight that opens it, not overlap it:
    /// otherwise the last sample of the day would end up at the same
    /// point as the first on chart and 3D map.
    #[test]
    fn ora_24_cade_a_fine_giornata() {
        let apertura = parse_timestamp("17/10/2026", "00:00:00").unwrap();
        let chiusura = parse_timestamp("17/10/2026", "24:00:00").unwrap();
        assert_eq!(chiusura - apertura, 24.0 * 3600.0);
    }

    /// Also with minutes and seconds, and with date formats other than the
    /// ones with the slash.
    #[test]
    fn ora_24_con_minuti_e_secondi() {
        let a = parse_timestamp("2026-10-17", "23:59:59").unwrap();
        let b = parse_timestamp("2026-10-17", "24:00:00").unwrap();
        assert_eq!(b - a, 1.0, "24:00:00 deve valere un secondo dopo le 23:59:59");

        let c = parse_timestamp("17-10-2026", "24:30:00").unwrap();
        let d = parse_timestamp("17-10-2026", "00:30:00").unwrap();
        assert_eq!(c - d, 24.0 * 3600.0);
    }

    /// Hour 24 present in the same cell as the date must be handled as
    /// in the archive with date and time joined in the first column.
    #[test]
    fn ora_24_nella_stessa_cella_della_data() {
        let a = parse_timestamp("2026-10-17 23:00:00", "").unwrap();
        let b = parse_timestamp("2026-10-17 24:00:00", "").unwrap();
        assert_eq!(b - a, 3600.0);
    }

    /// Ordinary hours must not be touched by the conversion.
    #[test]
    fn ore_ordinarie_invariate() {
        for h in ["00", "01", "09", "12", "21", "23"] {
            let t = parse_timestamp("2026-10-17", &format!("{h}:00:00")).unwrap();
            let dt = chrono::DateTime::from_timestamp(t as i64, 0).unwrap();
            assert_eq!(dt.format("%H:%M").to_string(), format!("{h}:00"), "ora {h}");
            assert_eq!(dt.format("%Y-%m-%d").to_string(), "2026-10-17", "ora {h}");
        }
    }

    /// A "24" that is not a time (year, file name, id) must not be converted.
    #[test]
    fn ventiquattro_non_orario_non_toccato() {
        // Year 2024 at the head of the date: the "24:" occurrence is not there,
        // but a year followed by a colon must never be rewritten.
        let t = parse_timestamp("2024-06-01", "10:00:00").unwrap();
        let dt = chrono::DateTime::from_timestamp(t as i64, 0).unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-06-01");
        assert_eq!(dt.format("%H:%M").to_string(), "10:00");

        // A "24" followed by a letter is not a time.
        let (testo, ore) = normalizza_ora_24("24abc");
        assert_eq!(testo, "24abc");
        assert_eq!(ore, 0.0);
    }

    /// A "24" mid-string, neither at the head nor after a space, is irrelevant.
    #[test]
    fn ventiquattro_in_posizione_non_oraria() {
        let (testo, ore) = normalizza_ora_24("sensore 24: interno");
        assert_eq!(testo, "sensore 24: interno");
        assert_eq!(ore, 0.0);
    }
}
