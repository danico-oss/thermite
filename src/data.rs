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

    // L'ora 24 designa in ISO 8601 la mezzanotte di fine giornata, ma la
    // libreria chrono ammette solo 00-23 e rifiuta il testo per intero. Senza
    // questa conversione la riga cadrebbe sul fallback «sola data», finendo
    // sovrapposta alla mezzanotte di inizio giorno: il campione non verrebbe
    // perso, ma scatterebbe di 24 ore a sinistra su grafico e mappa.
    // La conversione vale sia per la colonna ora sia per una cella unica che
    // contenga già «data ora», come documentato nel README.
    let (date_norm, ore_date) = normalizza_ora_24(date_str);
    let (time_norm, ore_time) = normalizza_ora_24(time_str);
    // In un file malformato entrambe le celle potrebbero portare «24»: si prende
    // una sola volta, mai raddoppiata.
    let ore_scarto = ore_date.max(ore_time);
    let date_norm = date_norm.as_str();
    let time_norm = time_norm.as_str();

    // La cella potrebbe contenere già "data ora" nella prima colonna.
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

/// Riscrive un orario `24:MM[:SS]` in `00:MM[:SS]` e restituisce i secondi da
/// sommare al risultato (86400, cioè 24 ore), oppure il testo invariato e `0.0`.
///
/// Il «24» va riconosciuto solo quando è in testa al testo o segue uno spazio,
/// così una data come `2024:...` o un nome sensore non vengono toccati.
fn normalizza_ora_24(testo: &str) -> (String, f64) {
    for (i, _) in testo.match_indices("24:") {
        let in_testa = i == 0;
        let dopo_spazio = i > 0 && testo.as_bytes()[i - 1] == b' ';
        if !(in_testa || dopo_spazio) {
            continue;
        }
        let resto = &testo[i + 3..];
        // Deve esserci almeno una cifra di minuto, altrimenti non è un orario.
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

/// Converte una cella in numero, tollerando la virgola come separatore decimale.
pub fn parse_number(value: &str) -> Option<f64> {
    let cleaned = value.trim().replace(',', ".");
    if cleaned.is_empty() {
        return None;
    }
    cleaned.parse::<f64>().ok()
}

/// Rileva il separatore di campo di un file CSV guardando la sua prima riga.
///
/// Conta le occorrenze di `,` e `;` che stanno **fuori** dalle virgolette e
/// restituisce la più frequente: così un file europeo separato da `;` con
/// virgola decimale (`01/06/2026;08:00;18,5`) viene letto correttamente.
///
/// A parità di conteggio vince la virgola, che è il separatore storico e quello
/// usato da `data/test.csv`. Se la riga è vuota restituisce comunque la
/// virgola, comportamento predefinito del crate `csv`.
///
/// L'analisi è sulle sole righe di intestazione: una riga di dati non viene mai
/// esaminata, quindi un campo testuale contente `;` non può ingannare il
/// rilevamento.
pub fn detect_separator(prima_riga: &str) -> u8 {
    let bytes = prima_riga.as_bytes();
    let mut virgole = 0usize;
    let mut punti_e_virgola = 0usize;
    let mut dentro_virgolette = false;
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            // Fine riga: il separatore non può trovarsi oltre.
            b'\n' | b'\r' => break,
            b'"' => {
                // Una coppia di virgolette dentro un campo è un escape (`""`),
                // non una chiusura.
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

    // --- Rilevamento del separatore di campo ---

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

    /// All'uguaglianza dei conteggi vince la virgola, separatore storico.
    #[test]
    fn separatore_pari_conteggi_vince_la_virgola() {
        assert_eq!(detect_separator("a,b;c"), b',');
    }

    /// Una riga vuota (o solo a capo) non deve scegliere a caso.
    #[test]
    fn separatore_riga_vuota() {
        assert_eq!(detect_separator(""), b',');
        assert_eq!(detect_separator("\n"), b',');
        assert_eq!(detect_separator("\r\n"), b',');
    }

    /// I separatori racchiusi tra virgolette appartengono al campo, non sono
    /// separatori: senza questo controllo un file separato da virgola con un
    /// campo che contiene un `;` verrebbe scambiato per file europeo.
    #[test]
    fn separatore_ignora_il_contenuto_delle_virgolette() {
        // Un `;` dentro un campo non deve prevalere: il separatore è la virgola.
        assert_eq!(detect_separator("\"Nota;urgente\",Ora,S1"), b',');
        // Idem una virgola dentro un campo di un file separato da `;`.
        assert_eq!(detect_separator("\"Nota,urgente\";Ora;S1"), b';');
        // Un campo vuoto racchiuso in virgolette non altera nulla.
        assert_eq!(detect_separator("\"\",Ora,S1"), b',');
    }

    /// Le virgolette doppie dentro un campo sono escape, non chiusura.
    #[test]
    fn separatore_virgolette_escape() {
        assert_eq!(detect_separator("\"dici \"\"sì\"\"\",Ora,S1"), b',');
    }

    /// La prima riga va oltre il terminatore di riga: un campo più a destra
    /// della testa non deve essere contato due volte o ignorato.
    #[test]
    fn separatore_si_ferma_alla_fine_riga() {
        assert_eq!(detect_separator("Data;Ora;S1\n2026-06-01,08:00,18.5"), b';');
    }

    /// Estremi realistici: header europeo con virgola decimale nei dati.
    #[test]
    fn separatore_file_europeo_completo() {
        let prima_riga = "Data;Ora;Sensore 1;Sensore 2";
        assert_eq!(detect_separator(prima_riga), b';');
    }

    // --- Orario di fine giornata ---

    /// `24:00:00` è la mezzanotte che chiude la giornata in ISO 8601. Deve
    /// valere 24 ore dopo la mezzanotte che la apre, non sovrapporsi ad essa:
    /// in caso contrario l'ultimo campione del giorno finirebbe nello stesso
    /// punto del primo su grafico e mappa 3D.
    #[test]
    fn ora_24_cade_a_fine_giornata() {
        let apertura = parse_timestamp("17/10/2026", "00:00:00").unwrap();
        let chiusura = parse_timestamp("17/10/2026", "24:00:00").unwrap();
        assert_eq!(chiusura - apertura, 24.0 * 3600.0);
    }

    /// Anche con minuti e secondi, e con formati di data diversi da quelli
    /// con la barra.
    #[test]
    fn ora_24_con_minuti_e_secondi() {
        let a = parse_timestamp("2026-10-17", "23:59:59").unwrap();
        let b = parse_timestamp("2026-10-17", "24:00:00").unwrap();
        assert_eq!(b - a, 1.0, "24:00:00 deve valere un secondo dopo le 23:59:59");

        let c = parse_timestamp("17-10-2026", "24:30:00").unwrap();
        let d = parse_timestamp("17-10-2026", "00:30:00").unwrap();
        assert_eq!(c - d, 24.0 * 3600.0);
    }

    /// L'ora 24 presente nella stessa cella della data va trattata come
    /// nell'archivio con data e ora unite nella prima colonna.
    #[test]
    fn ora_24_nella_stessa_cella_della_data() {
        let a = parse_timestamp("2026-10-17 23:00:00", "").unwrap();
        let b = parse_timestamp("2026-10-17 24:00:00", "").unwrap();
        assert_eq!(b - a, 3600.0);
    }

    /// Le ore ordinarie non devono essere toccate dalla conversione.
    #[test]
    fn ore_ordinarie_invariate() {
        for h in ["00", "01", "09", "12", "21", "23"] {
            let t = parse_timestamp("2026-10-17", &format!("{h}:00:00")).unwrap();
            let dt = chrono::DateTime::from_timestamp(t as i64, 0).unwrap();
            assert_eq!(dt.format("%H:%M").to_string(), format!("{h}:00"), "ora {h}");
            assert_eq!(dt.format("%Y-%m-%d").to_string(), "2026-10-17", "ora {h}");
        }
    }

    /// Un «24» che non è un'ora (anno, nome file, id) non deve essere convertito.
    #[test]
    fn ventiquattro_non_orario_non_toccato() {
        // Anno 2024 in testa alla data: l'occorrenza "24:" non c'è, ma un anno
        // seguito da due punti non deve mai essere riscritto.
        let t = parse_timestamp("2024-06-01", "10:00:00").unwrap();
        let dt = chrono::DateTime::from_timestamp(t as i64, 0).unwrap();
        assert_eq!(dt.format("%Y-%m-%d").to_string(), "2024-06-01");
        assert_eq!(dt.format("%H:%M").to_string(), "10:00");

        // Un "24" seguito da una lettera non è un orario.
        let (testo, ore) = normalizza_ora_24("24abc");
        assert_eq!(testo, "24abc");
        assert_eq!(ore, 0.0);
    }

    /// Un «24» a metà stringa, non in testa né dopo spazio, è irrilevante.
    #[test]
    fn ventiquattro_in_posizione_non_oraria() {
        let (testo, ore) = normalizza_ora_24("sensore 24: interno");
        assert_eq!(testo, "sensore 24: interno");
        assert_eq!(ore, 0.0);
    }
}
