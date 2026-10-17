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

## Formato del file dati

Il file (`.csv`, `.txt`, `.xlsx`, `.xls`) deve essere organizzato **una riga per
campionamento**, con la **prima riga riservata alle intestazioni**:

| Colonna | Contenuto | Note |
| --- | --- | --- |
| 1 | **Data** | vedi i formati ammessi sotto |
| 2 | **Ora** | opzionale se la colonna 1 contiene già data e ora |
| 3 | **Sensore 1** | valore numerico |
| 4 | **Sensore 2** | valore numerico |
| *n* | ***Sensore n-2*** | si continua all'infinito |

Le colonne dalla terza in poi sono tutte serie di sensori: ce ne possono essere
due come venti, senza limite. **I nomi delle colonne 3, 4, … diventano i nomi
dei sensori** e vengono usati per le etichette del grafico e per la mappa 3D,
dove ogni sensore va posizionato sul container. Le due celle di intestazione
delle colonne 1 e 2 vengono ignorate.

Esempio con separatore virgola (`data/test.csv`):

```csv
Data,Ora,Sensore_1,Sensore_2
2026-06-01,08:00:00,18.5,19.2
2026-06-01,09:00:00,20.1,21.0
2026-06-01,10:00:00,22.4,22.8
```

### Separatore di campo

Si accettano sia la **virgola `,`** che il **punto e virgola `;`**, rilevati
automaticamente dalla prima riga del file: non va indicato nulla. Un file
europeo classico, con `;` come separatore e virgola decimale, si legge senza
conversioni:

```csv
Data;Ora;Sensore_1;Sensore_2
2026-06-01;08:00:00;18,5;19,2
2026-06-01;09:00:00;20,1;21,0
```

Il rilevamento conta i separatori **fuori** dalle virgolette sulla sola riga di
intestazione, quindi un campo testuale che contenga una virgola o un `;` non
può ingannarlo. A parità di conteggi vince la virgola.

### Format di data e ora

La prima e la seconda colonna vengono ricombinate in «data ora» e provate contro
una lista di formati; il primo che corrisponde vince.

Con data e ora separate in due colonne:

| Esempio | Formato |
| --- | --- |
| `2026-06-01` + `08:00:00` | `%Y-%m-%d %H:%M:%S` |
| `01/06/2026` + `08:00:00` | `%d/%m/%Y %H:%M:%S` |
| `2026-06-01` + `08:00` | `%Y-%m-%d %H:%M` |
| `01/06/2026` + `08:00` | `%d/%m/%Y %H:%M` |
| `2026/06/01` + `08:00:00` | `%Y/%m/%d %H:%M:%S` |
| `01-06-2026` + `08:00:00` | `%d-%m-%Y %H:%M:%S` |
| `2026-06-01` + `08:00:00.500` | `%Y-%m-%d %H:%M:%S%.f` (frazionali) |
| `01/06/2026` + `08:00:00.500` | `%d/%m/%Y %H:%M:%S%.f` (frazionali) |

Con la sola colonna 1 (la 2 vuota), accetta anche la sola data, a patto che sia
in uno di questi formati, e l'ora viene impostata a mezzanotte:

`2026-06-01` · `01/06/2026` · `01-06-2026` · `2026/06/01`

### Valori numerici

- Sono accettati sia il punto che la **virgola decimale**: `18.5` e `18,5`
  leggono lo stesso valore.
- Le celle vuote o non numeriche vengono saltate per quel sensore, senza
  bloccare le altre serie.
- Le righe con data/ora non riconoscibili vengono ignorate del tutto.

### Limiti da ricordare

- La prima riga è sempre interpretata come intestazione, quindi non va omessa e
  non deve contenere dati.
- Per i file Excel viene usato il primo foglio.
- Il separatore può essere `,` o `;`, ma **non il tabulatore**: un file TSV va
  convertito.

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
