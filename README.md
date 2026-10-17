# Thermite

**Mappatura Termica di Trasporto**

Applicazione desktop in Rust per l'analisi dei dati dei sensori di temperatura
durante il trasporto. Importa file CSV ed Excel, produce grafici temporali e una
mappa 3D interattiva del container con i sensori posizionati sulle facce.

## Funzionalità

- **Gestione progetti**: creazione, modifica ed eliminazione; ogni progetto
  definisce tipo di container e intervallo di temperature.
- **Importazione dati**: file CSV/TXT ed Excel (`.xlsx`, `.xls`) con rilevamento
  automatico del formato data/ora.
- **Griglia dati**: visualizzazione tabellare con scorrimento sia orizzontale
  sia verticale.
- **Grafici temporali**: una serie per sensore, con tooltip e valori min/max.
- **Mappa 3D**: parallelepipedo ruotabile che rappresenta il container, con i
  sensori posizionati sulle facce secondo i dati del progetto. Container
  semitrasparente, selezione dei sensori, anteprima ed export PDF.
- **Integrità dei dati**: al salvataggio del progetto viene calcolato il CRC32
  del file dati; al caricamento il codice viene riverificato e un file
  modificato viene rifiutato.
- **Interfaccia**: tema chiaro/scuro, stile compatto per notebook, lingue
  italiano / inglese / portoghese.

## Compilazione

Richiede [Rust](https://rustup.rs/) con toolchain MSVC su Windows (oppure
qualunque piattaforma supportata da `eframe`).

```sh
cargo build --release
```

L'eseguibile si trova in `target/release/thermite.exe`.

### Risorsa di versione su Windows

`build.rs` incorpora `assets/thermite.ico` e un blocco VERSIONINFO
(FileDescription, ProductName, versione, licenza) nell'eseguibile usando
`rc.exe` dello Windows SDK. Se `rc.exe` non viene trovato la compilazione
prosegue normalmente, senza la sola risorsa di versione.

## Esecuzione

```sh
cargo run --release
```

I dati vengono salvati in `advanced_projects.db` (SQLite) e le preferenze in
`thermite_settings.txt`, entrambi nella cartella di lavoro. Nessuno dei due va
versionato.

## Struttura

| Percorso | Contenuto |
| --- | --- |
| `src/main.rs` | punto d'ingresso, gestione comandi |
| `src/data.rs` | parsing CSV/Excel e serie temporali |
| `src/crc32.rs` | CRC32 IEEE 802.3 del file dati |
| `src/models.rs` | modelli e accesso al database SQLite |
| `src/theme.rs` | palette e stili dei due temi |
| `src/i18n.rs` | tabelle di traduzione IT / EN / PT-BR |
| `src/scene3d.rs` | geometria e proiezione 3D della scena |
| `src/icon.rs` | icona disegnata proceduralmente |
| `src/views/` | schede: Gestione, Ricerca, Dati, Grafico, Mappa 3D, Configurazioni |
| `src/export.rs` | esportazione PDF di grafici e mappa |

## Test

```sh
cargo test
```

Coprono parsing, geometria 3D, CRC32, migrazione del database, esportazione PDF
e disegno dell'icona.

## Licenza

GNU GENERAL PUBLIC LICENSE
Version 3, 29 June 2007
