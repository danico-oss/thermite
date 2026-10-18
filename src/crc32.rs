//! CRC32 (IEEE 802.3 polynomial, the same one as zip/PNG) of the data files.
//!
//! It serves to detect whether a project's data file has been modified after
//! being saved: the code computed at save time is kept in the database
//! (`projects.data_crc32`) and compared before loading the data.
//!
//! Table-based implementation (32 entries) with no external dependencies.

use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Remainder table, built only once at runtime.
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

/// Initial register of the CRC-32 standard (later finalized with the final XOR).
const INIZIALE: u32 = 0xFFFF_FFFF;
/// Finalization mask of the CRC-32 standard.
const FINALE: u32 = 0xFFFF_FFFF;

/// Continues the CRC from an intermediate register: it is the _raw_ register,
/// without the finalization, so pass it to `finale()` only on the last block.
///
/// The starting value is `INIZIALE`, not `0`: concatenating the pieces of a file
/// produces the same result as the whole file.
pub fn bytes_con(parziale: u32, dati: &[u8]) -> u32 {
    let t = tabella();
    let mut c = parziale;
    for &b in dati {
        c = t[((c ^ b as u32) & 0xFF) as usize] ^ (c >> 8);
    }
    c
}

/// Applies the standard's final inversion to a raw register.
fn finale(registro: u32) -> u32 {
    registro ^ FINALE
}

/// CRC32 of a file, read in blocks (it also works on large files).
///
/// Returns `None` if the file does not exist or is not readable.
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

/// Formats a CRC as an 8-digit uppercase hexadecimal string.
pub fn esadecimale(crc: u32) -> String {
    format!("{crc:08X}")
}

/// CRC32 of a file as a hexadecimal string, or `None` if it is not readable.
///
/// Used when saving the project: `None` is written as `NULL` in the
/// database, so the check at load time is simply skipped (the previous
/// behavior).
pub fn file_hex(percorso: impl AsRef<Path>) -> Option<String> {
    file(percorso).map(esadecimale)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CRC32 of a byte buffer, using the same flow as `file()`.
    fn bytes(dati: &[u8]) -> u32 {
        finale(bytes_con(INIZIALE, dati))
    }

    /// The known reference values for the CRC-32 standard (polynomial
    /// 0xEDB88320): the same ones returned by zlib, PNG and zip.
    #[test]
    fn valori_di_riferimento() {
        assert_eq!(bytes(b""), 0x0000_0000);
        assert_eq!(bytes(b"123456789"), 0xCBF4_3926);
        assert_eq!(bytes(b"a"), 0xE8B7_BE43);
        assert_eq!(bytes(b"Thermite"), 0x2FFD_FBC5);
    }

    /// Splitting the stream into chunks must not change the result.
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

    /// The CRC of a temporary file matches that of its content.
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

    /// A file larger than one block (64 KiB) is read in full.
    #[test]
    fn file_grande_oltre_un_blocco() {
        let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".to_string());
        let p = std::path::Path::new(&dir).join("thermite_crc32_grande.csv");
        let dati: Vec<u8> = (0..200_000u32).map(|i| b'a' + (i % 26) as u8).collect();
        std::fs::write(&p, &dati).unwrap();
        assert_eq!(file(&p), Some(bytes(&dati)));
        let _ = std::fs::remove_file(&p);
    }

    /// A nonexistent path produces `None`, not a panic.
    #[test]
    fn percorso_assente() {
        assert_eq!(file("C:\\percorso\\inesistente\\thermite.csv"), None);
        assert_eq!(file_hex("C:\\percorso\\inesistente\\thermite.csv"), None);
    }

    /// The hexadecimal always has eight digits, with the leading zero.
    #[test]
    fn esadecimale_otto_cifre() {
        assert_eq!(esadecimale(0x0000_000A), "0000000A");
        assert_eq!(esadecimale(0xCBF4_3926), "CBF43926");
    }
}
