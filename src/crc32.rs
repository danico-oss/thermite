//! CRC32 (polinomio IEEE 802.3, lo stesso di zip/PNG) dei file dati.
//!
//! Serve a rilevare se il file dati di un progetto è stato modificato dopo il
//! salvataggio: il codice calcolato al momento del salvataggio viene tenuto nel
//! database (`projects.data_crc32`) e confrontato prima di caricare i dati.
//!
//! Implementazione a tabella (32 voci) senza dipendenze esterne.

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Tabella dei rimanenti, costruita una sola volta a runtime.
fn tabella() -> [u32; 256] {
    let mut t = [0u32; 256];
    for (i, v) in t.iter_mut().enumerate() {
        let mut c = i as u32;
        for _ in 0..8 {
            c = if c & 1 == 1 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
        }
        *v = c;
    }
    t
}

/// Registro iniziale dello standard CRC-32 (poi finalizzato con XOR finale).
const INIZIALE: u32 = 0xFFFF_FFFF;
/// Maschera di finalizzazione dello standard CRC-32.
const FINALE: u32 = 0xFFFF_FFFF;

/// Continua il CRC da un registro intermedio: è il registro _grezzo_, senza la
/// finalizzazione, quindi va passato a `finale()` solo all'ultimo blocco.
///
/// Il valore di partenza è `INIZIALE`, non `0`: concatenare i pezzi di un file
/// produce lo stesso risultato del file intero.
pub fn bytes_con(parziale: u32, dati: &[u8]) -> u32 {
    let t = tabella();
    let mut c = parziale;
    for &b in dati {
        c = t[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c
}

/// Applica l'inversione finale dello standard a un registro grezzo.
fn finale(registro: u32) -> u32 {
    registro ^ FINALE
}

/// CRC32 di un file, letto a blocchi (funziona anche su file grandi).
///
/// Restituisce `None` se il file non esiste o non è leggibile.
pub fn file(percorso: impl AsRef<Path>) -> Option<u32> {
    let mut f = File::open(percorso).ok()?;
    let mut buf = vec![0u8; 64 * 1024];
    let mut c = INIZIALE;
    loop {
        let n = f.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        c = bytes_con(c, &buf[..n]);
    }
    Some(finale(c))
}

/// Formatta un CRC come stringa esadecimale maiuscola a 8 cifre.
pub fn esadecimale(crc: u32) -> String {
    format!("{crc:08X}")
}

/// CRC32 di un file come stringa esadecimale, o `None` se non leggibile.
///
/// Usato al salvataggio del progetto: `None` viene scritto come `NULL` nel
/// database, quindi la verifica in fase di caricamento viene semplicemente
/// saltata (comportamento di prima).
pub fn file_hex(percorso: impl AsRef<Path>) -> Option<String> {
    file(percorso).map(esadecimale)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CRC32 di un buffer di byte, usando lo stesso flusso di `file()`.
    fn bytes(dati: &[u8]) -> u32 {
        finale(bytes_con(INIZIALE, dati))
    }

    /// I valori noti di riferimento per lo standard CRC-32 (polinomio
    /// 0xEDB88320): gli stessi che restituiscono zlib, PNG e zip.
    #[test]
    fn valori_di_riferimento() {
        assert_eq!(bytes(b""), 0x0000_0000);
        assert_eq!(bytes(b"123456789"), 0xCBF4_3926);
        assert_eq!(bytes(b"a"), 0xE8B7_BE43);
        assert_eq!(bytes(b"Thermite"), 0x2FFD_FBC5);
    }

    /// La frammentazione del flusso non deve cambiare il risultato.
    #[test]
    fn indipendente_dai_blocchi() {
        let dati = b"0123456789abcdef0123456789abcdef";
        let intero = bytes(dati);
        let mut parziale = INIZIALE;
        parziale = bytes_con(parziale, &dati[..7]);
        parziale = bytes_con(parziale, &dati[7..20]);
        parziale = bytes_con(parziale, &dati[20..]);
        assert_eq!(finale(parziale), intero);
    }

    /// Il CRC di un file temporaneo coincide con quello del suo contenuto.
    #[test]
    fn file_tempaneo() {
        let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".to_string());
        let p = std::path::Path::new(&dir).join("thermite_crc32_prova.csv");
        let contenuto = b"Data,Ora,S1\n2024-01-01,10:00,21.5\n";
        std::fs::write(&p, contenuto).unwrap();
        let atteso = bytes(contenuto);
        assert_eq!(file(&p), Some(atteso));
        assert_eq!(file_hex(&p).as_deref(), Some(esadecimale(atteso).as_str()));
        let _ = std::fs::remove_file(&p);
    }

    /// Un file più grande di un blocco (64 KiB) viene letto per intero.
    #[test]
    fn file_grande_oltre_un_blocco() {
        let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".to_string());
        let p = std::path::Path::new(&dir).join("thermite_crc32_grande.csv");
        let dati: Vec<u8> = (0..200_000u32).map(|i| b'a' + (i % 26) as u8).collect();
        std::fs::write(&p, &dati).unwrap();
        assert_eq!(file(&p), Some(bytes(&dati)));
        let _ = std::fs::remove_file(&p);
    }

    /// Un percorso inesistente produce `None`, non un panic.
    #[test]
    fn percorso_assente() {
        assert_eq!(file("C:\\percorso\\inesistente\\thermite.csv"), None);
        assert_eq!(file_hex("C:\\percorso\\inesistente\\thermite.csv"), None);
    }

    /// L'esadecimale ha sempre otto cifre, con lo zero iniziale.
    #[test]
    fn esadecimale_otto_cifre() {
        assert_eq!(esadecimale(0x0000_000A), "0000000A");
        assert_eq!(esadecimale(0xCBF4_3926), "CBF43926");
    }
}
