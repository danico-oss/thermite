//! Icona del programma: un cubo isometrico (il container del trasporto) con
//! un gradiente termico che va dal blu "freddo" in basso a sinistra al rosso
//! "caldo" in alto a destra.
//!
//! Il disegno è procedurale (nessuna libreria di immagini, nessun file
//! esterno): la stessa funzione produce i pixel della finestra e il file
//! `assets/thermite.ico` usato per l'icona del .exe.

use eframe::egui;

/// Blu "freddo" del gradiente termico.
const FREDDO: [f32; 3] = [36.0, 104.0, 216.0];
/// Rosso "caldo" del gradiente termico.
const CALDO: [f32; 3] = [238.0, 62.0, 38.0];
/// Luminosità relativa delle tre facce del cubo: [alto, sinistra, destra].
const FACCE: [f32; 3] = [1.12, 0.86, 0.62];
/// Fattore di supercampionamento: i bordi restano nitidi senza antialiasing
/// esterno, poi l'immagine viene ridotta con filtro a scatola.
const SS: u32 = 4;

/// Icona da passare a `ViewportBuilder::with_icon`.
pub fn egui_icon(size: u32) -> egui::IconData {
    egui::IconData {
        rgba: render(size),
        width: size,
        height: size,
    }
}

/// Pixel RGBA (canale alfa non premultiplicato) dell'icona a `size`×`size`.
pub fn render(size: u32) -> Vec<u8> {
    let size = size.max(8);
    let big = size * SS;
    let mut buf = vec![0u8; (big * big * 4) as usize];
    disegna(&mut buf, big);
    riduci(&buf, big, size)
}

/// Temperatura di un punto: `0.0` = freddo, `1.0` = caldo. Il gradiente corre
/// in diagonale, dal basso a sinistra (freddo) all'alto a destra (caldo).
fn temperatura(px: f32, py: f32) -> f32 {
    ((px + (1.0 - py)) / 2.0).clamp(0.0, 1.0)
}

/// Disegna il cubo su una tela `n`×`n` trasparente (coordinate normalizzate).
fn disegna(buf: &mut [u8], n: u32) {
    // Vertici del cubo isometrico.
    let (cx, alto, a, b, c) = (0.5, 0.085, 0.40, 0.19, 0.44);
    let vertici = [
        [cx, alto],             // cima
        [cx + a, alto + b],     // alto destra
        [cx, alto + 2.0 * b],   // centro (vertice medio basso della faccia alta)
        [cx - a, alto + b],     // alto sinistra
        [cx - a, alto + b + c], // basso sinistra
        [cx, alto + 2.0 * b + c], // basso centro
        [cx + a, alto + b + c], // basso destra
    ];
    let [t, r, m, l, lb, bb, rb] = vertici;
    // Le tre facce visibili: alta, sinistra, destra (tutte in senso orario).
    let facce = [[t, r, m, l], [l, m, bb, lb], [m, r, rb, bb]];

    let f = n as f32;
    for y in 0..n {
        for x in 0..n {
            let px = (x as f32 + 0.5) / f;
            let py = (y as f32 + 0.5) / f;
            let temp = temperatura(px, py);
            let base = [
                lerp(FREDDO[0], CALDO[0], temp),
                lerp(FREDDO[1], CALDO[1], temp),
                lerp(FREDDO[2], CALDO[2], temp),
            ];
            for (i, q) in facce.iter().enumerate() {
                if dentro_quad(px, py, q) {
                    let lum = FACCE[i];
                    scrivi(
                        buf,
                        n,
                        x,
                        y,
                        [
                            (base[0] * lum).min(255.0) as u8,
                            (base[1] * lum).min(255.0) as u8,
                            (base[2] * lum).min(255.0) as u8,
                            255,
                        ],
                    );
                    break;
                }
            }
        }
    }
}

fn scrivi(buf: &mut [u8], n: u32, x: u32, y: u32, rgba: [u8; 4]) {
    let o = ((y * n + x) * 4) as usize;
    buf[o..o + 4].copy_from_slice(&rgba);
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Punto dentro un quadrilatero convesso (test dei prodotti vettoriali).
fn dentro_quad(px: f32, py: f32, q: &[[f32; 2]; 4]) -> bool {
    let mut pos = false;
    let mut neg = false;
    for i in 0..4 {
        let a = q[i];
        let b = q[(i + 1) % 4];
        let cr = (b[0] - a[0]) * (py - a[1]) - (b[1] - a[1]) * (px - a[0]);
        if cr > 0.0 {
            pos = true;
        } else if cr < 0.0 {
            neg = true;
        }
        if pos && neg {
            return false;
        }
    }
    true
}

/// Riduzione con filtro a scatola: media pesata per l'alfa, così i bordi non
/// diventano scuri quando l'immagine viene vista su sfondo chiaro.
fn riduci(buf: &[u8], big: u32, size: u32) -> Vec<u8> {
    let s = (big / size) as usize;
    let mut out = vec![0u8; (size * size * 4) as usize];
    for y in 0..size as usize {
        for x in 0..size as usize {
            let (mut ra, mut ga, mut ba, mut aa) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
            for j in 0..s {
                for i in 0..s {
                    let o = (((y * s + j) as u32 * big + (x * s + i) as u32) * 4) as usize;
                    let a = buf[o + 3] as f32;
                    ra += buf[o] as f32 * a;
                    ga += buf[o + 1] as f32 * a;
                    ba += buf[o + 2] as f32 * a;
                    aa += a;
                }
            }
            let k = (s * s) as f32;
            let o = (y * size as usize + x) * 4;
            if aa > 0.0 {
                out[o] = (ra / aa).round() as u8;
                out[o + 1] = (ga / aa).round() as u8;
                out[o + 2] = (ba / aa).round() as u8;
                out[o + 3] = (aa / k).round() as u8;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// Taglie inserite nel file `.ico` (come da pratica corrente su Windows).
    const TAGLIE: [u32; 6] = [16, 24, 32, 48, 64, 256];

    /// Scrive `assets/thermite.ico` (rigenerato ogni volta che girano i test):
    /// serve al `build.rs` per incorporare l'icona nel file .exe.
    #[test]
    fn scrivi_file_ico() {
        std::fs::create_dir_all("assets").expect("cartella assets");
        let bytes = costruisci_ico(&TAGLIE);
        assert_eq!(&bytes[0..4], &[0, 0, 1, 0], "intestazione ICO non valida");
        std::fs::write("assets/thermite.ico", &bytes).expect("scrittura assets/thermite.ico");
    }

    /// Controllo che le voci dell'ico siano coerenti (offset e dimensioni).
    #[test]
    fn ico_interna_coerente() {
        let bytes = costruisci_ico(&TAGLIE);
        let n = u16::from_le_bytes([bytes[4], bytes[5]]) as usize;
        assert_eq!(n, TAGLIE.len());
        for i in 0..n {
            let e = 6 + i * 16;
            let (dim, off) = (
                u32::from_le_bytes(bytes[e + 8..e + 12].try_into().unwrap()) as usize,
                u32::from_le_bytes(bytes[e + 12..e + 16].try_into().unwrap()) as usize,
            );
            assert!(off + dim <= bytes.len(), "voce {i} fuori dal file");
            let s = TAGLIE[i] as usize;
            assert_eq!(dim, 40 + s * s * 4 + ((s + 31) / 32 * 4) * s, "voce {i}");
        }
    }

    /// Genera un "foglio di contatto" PNG da guardare per verificare il
    /// disegno sia su sfondo chiaro sia su sfondo scuro, e salva anche
    /// `assets/thermite.png` a 256 px come riferimento.
    #[test]
    fn anteprime_png() {
        std::fs::create_dir_all("assets").expect("cartella assets");
        let (tela, w, h) = foglio(&[256, 64, 48, 32, 16]);
        let dir = std::env::var("TEMP").unwrap_or_else(|_| ".".to_string());
        scrivi_png(&std::path::Path::new(&dir).join("thermite_icona_contatto.png"), &tela, w, h);
        let rgba = render(256);
        scrivi_png(Path::new("assets/thermite.png"), &rgba, 256, 256);
    }

    /// Campiona alcuni pixel per verificare che la geometria sia quella
    /// voluta: fuori dal cubo è trasparente, le tre facce sono opache, la
    /// faccia sinistra è fredda (blu), quella destra più calda, e la
    /// luminosità decresce da alto → sinistra → destra.
    #[test]
    fn geometria_cubo() {
        let s = 256u32;
        let rgba = render(s);
        let px = |nx: f32, ny: f32| -> [u8; 4] {
            let o = ((ny * s as f32) as u32 * s + (nx * s as f32) as u32) as usize * 4;
            [rgba[o], rgba[o + 1], rgba[o + 2], rgba[o + 3]]
        };
        let calore = |p: [u8; 4]| p[0] as i32 - p[2] as i32;
        let lum = |p: [u8; 4]| p[0] as u32 + p[1] as u32 + p[2] as u32;

        // Fuori dal cubo: trasparente.
        for (x, y) in [(0.03, 0.03), (0.97, 0.03), (0.03, 0.97), (0.50, 0.99)] {
            let p = px(x, y);
            assert_eq!(p[3], 0, "trasparente a ({x}, {y}): {p:?}");
        }
        // Le tre facce: opache.
        let alta = px(0.50, 0.27);
        let sinistra = px(0.25, 0.60);
        let destra = px(0.75, 0.60);
        for (nome, p) in [("alta", alta), ("sinistra", sinistra), ("destra", destra)] {
            assert_eq!(p[3], 255, "faccia {nome} opaca: {p:?}");
        }
        // Calore: sinistra fredda, destra e alta più calde.
        assert!(calore(sinistra) < 0, "sinistra fredda: {sinistra:?}");
        assert!(calore(alta) > calore(sinistra), "alta più calda di sinistra");
        assert!(
            calore(destra) > calore(sinistra),
            "destra più calda di sinistra"
        );
        // Luminosità: alto > sinistra > destra (facce diverse).
        assert!(lum(alta) > lum(sinistra), "alta più luminosa: {alta:?}");
        assert!(
            lum(sinistra) > lum(destra),
            "sinistra più chiara di destra: {sinistra:?} vs {destra:?}"
        );
    }

    // --- costruzione file -------------------------------------------------

    fn costruisci_ico(taglie: &[u32]) -> Vec<u8> {
        let voci: Vec<(u32, Vec<u8>)> = taglie
            .iter()
            .map(|&s| (s, dati_bmp_ico(&render(s), s)))
            .collect();
        let n = voci.len() as u16;
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(&0u16.to_le_bytes()); // riservato
        out.extend_from_slice(&1u16.to_le_bytes()); // tipo = icone
        out.extend_from_slice(&n.to_le_bytes());
        let mut offset = 6 + 16 * n as u32;
        for (s, data) in &voci {
            let b = if *s >= 256 { 0u8 } else { *s as u8 };
            out.push(b); // larghezza (0 = 256)
            out.push(b); // altezza
            out.push(0); // colori in palette
            out.push(0); // riservato
            out.extend_from_slice(&1u16.to_le_bytes()); // planes
            out.extend_from_slice(&32u16.to_le_bytes()); // bit/pixel
            out.extend_from_slice(&(data.len() as u32).to_le_bytes());
            out.extend_from_slice(&offset.to_le_bytes());
            offset += data.len() as u32;
        }
        for (_, data) in &voci {
            out.extend_from_slice(data);
        }
        out
    }

    /// Voce ICO in formato BMP 32 bit (XOR + maschera AND).
    fn dati_bmp_ico(rgba: &[u8], s: u32) -> Vec<u8> {
        let passo_maschera = ((s + 31) / 32) * 4;
        let xor = s * s * 4;
        let totale = xor + passo_maschera * s;
        let mut d: Vec<u8> = Vec::with_capacity(40 + totale as usize);
        d.extend_from_slice(&40u32.to_le_bytes()); // dimensione BITMAPINFOHEADER
        d.extend_from_slice(&(s as i32).to_le_bytes());
        d.extend_from_slice(&((s * 2) as i32).to_le_bytes()); // altezza doppia
        d.extend_from_slice(&1u16.to_le_bytes()); // planes
        d.extend_from_slice(&32u16.to_le_bytes()); // bit/pixel
        d.extend_from_slice(&0u32.to_le_bytes()); // compressione BI_RGB
        d.extend_from_slice(&totale.to_le_bytes());
        d.extend_from_slice(&0i32.to_le_bytes()); // dpi x
        d.extend_from_slice(&0i32.to_le_bytes()); // dpi y
        d.extend_from_slice(&0u32.to_le_bytes()); // colori in palette
        d.extend_from_slice(&0u32.to_le_bytes()); // colori importanti
        for y in (0..s).rev() {
            for x in 0..s {
                let o = ((y * s + x) * 4) as usize;
                d.push(rgba[o + 2]); // B
                d.push(rgba[o + 1]); // G
                d.push(rgba[o]); // R
                d.push(rgba[o + 3]); // A (maschera AND: basta il canale alfa)
            }
        }
        d.extend(std::iter::repeat_n(0u8, totale as usize - xor as usize)); // AND a zero
        d
    }

    // --- anteprime --------------------------------------------------------

    /// Compone icone a grandezze diverse su due metà: bianco sopra, grigio
    /// scuro sotto. Restituisce la tela RGBA (larghezza, altezza).
    fn foglio(taglie: &[u32]) -> (Vec<u8>, u32, u32) {
        let w = 16 + taglie.iter().sum::<u32>() + 8 * (taglie.len() as u32 - 1);
        let riga = 16 + 256 + 8;
        let h = riga * 2;
        let mut tela = vec![0u8; (w * h * 4) as usize];
        let (chiaro, scuro) = ([255u8; 3], [44u8, 46, 50]);
        for y in 0..h {
            let sfondo = if y < riga { chiaro } else { scuro };
            for x in 0..w {
                let o = ((y * w + x) * 4) as usize;
                tela[o..o + 3].copy_from_slice(&sfondo);
                tela[o + 3] = 255;
            }
        }
        let mut x = 8;
        for &s in taglie {
            let rgba = render(s);
            for (metà, sfondo) in [chiaro, scuro].into_iter().enumerate() {
                let y0 = metà as u32 * riga + 8 + (256 - s) / 2;
                for y in 0..s {
                    for xx in 0..s {
                        let o = ((y * s + xx) * 4) as usize;
                        let a = rgba[o + 3] as u32;
                        let inv = 255 - a;
                        let p = (((y0 + y) * w + x + xx) * 4) as usize;
                        for k in 0..3 {
                            let v = (rgba[o + k] as u32 * a + sfondo[k] as u32 * inv) / 255;
                            tela[p + k] = v as u8;
                        }
                    }
                }
            }
            x += s + 8;
        }
        (tela, w, h)
    }

    // --- PNG (deflate con blocchi "stored": niente dipendenze) ------------

    fn scrivi_png(path: &Path, rgba: &[u8], w: u32, h: u32) {
        // Filtro "None" su ogni riga, poi zlib con blocchi non compressi.
        let mut grezzo = Vec::with_capacity((w * h * 4 + h) as usize);
        for y in 0..h {
            grezzo.push(0);
            grezzo.extend(&rgba[(y * w * 4) as usize..((y + 1) * w * 4) as usize]);
        }
        let mut idat = vec![0x78, 0x01];
        let mut i = 0;
        while i < grezzo.len() {
            let n = (grezzo.len() - i).min(0xFFFF);
            let ultimo = i + n == grezzo.len();
            idat.push(if ultimo { 1 } else { 0 }); // BFINAL + BTYPE=00
            idat.extend(&(n as u16).to_le_bytes());
            idat.extend(&(!(n as u16)).to_le_bytes());
            idat.extend(&grezzo[i..i + n]);
            i += n;
        }
        idat.extend(&adler32(&grezzo).to_be_bytes());

        let mut out: Vec<u8> = Vec::new();
        out.extend(&[0x89, b'P', b'N', b'G', 13, 10, 26, 10]);
        let mut ihdr = Vec::new();
        ihdr.extend(&w.to_be_bytes());
        ihdr.extend(&h.to_be_bytes());
        ihdr.extend(&[8, 6, 0, 0, 0]); // 8 bit, RGBA
        blocco_png(&mut out, b"IHDR", &ihdr);
        blocco_png(&mut out, b"IDAT", &idat);
        blocco_png(&mut out, b"IEND", &[]);
        std::fs::write(path, out).expect("scrittura anteprima");
    }

    fn blocco_png(out: &mut Vec<u8>, tipo: &[u8; 4], dati: &[u8]) {
        out.extend(&(dati.len() as u32).to_be_bytes());
        out.extend(tipo);
        out.extend(dati);
        let crc = crc32_tabellato();
        let mut c = 0xFFFF_FFFFu32;
        for b in tipo.iter().chain(dati.iter()) {
            c = crc[(*b ^ c as u8) as usize] ^ (c >> 8);
        }
        out.extend((c ^ 0xFFFF_FFFF).to_be_bytes());
    }

    fn crc32_tabellato() -> [u32; 256] {
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

    fn adler32(dati: &[u8]) -> u32 {
        let (mut a, mut b) = (1u32, 0u32);
        for &x in dati {
            a = (a + x as u32) % 65521;
            b = (b + a) % 65521;
        }
        (b << 16) | a
    }
}
