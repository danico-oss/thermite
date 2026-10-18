# Thermite

**Thermal Transport Mapping**

Desktop application in Rust for the analysis of temperature sensor data during
transport. It imports CSV and Excel files, produces time charts and an
interactive 3D map of the container with the sensors positioned on the faces.

## Features

- **Project management**: creation, editing and deletion; every project defines
  the container type and the temperature range.
- **Data import**: CSV/TXT and Excel (`.xlsx`, `.xls`) files with automatic
  detection of the date/time format.
- **Data grid**: table view with both horizontal and vertical scrolling.
- **Project list**: list with a search filter; a click on a row selects it and a
  double click opens the data, exactly like the "Open data" button of the same
  row.
- **Time charts**: one series per sensor, with tooltips and min/max values.
- **3D map**: rotatable parallelepiped that represents the container, with the
  sensors positioned on the faces according to the project data. Semi-
  transparent container, sensor selection, preview and PDF export. A sensor that
  has already been placed is dragged **freely through space**: the point follows
  the cursor while keeping its distance from the camera and, looking at the
  container from a three-quarter view, it moves with components on all three
  axes x, y and z instead of only the two of the wall, so it can even detach
  from the container. The new position is saved to the database when the button
  is released; to put a sensor back on a wall it is enough to remove it (right
  click on the point) and place it again with a click.
  Next to the removal icon there is an **eye** that hides and shows the sensor
  in the diagram: a hidden sensor does not appear in the scene, cannot be
  grabbed with the pointer and stays excluded from the preview and the PDF as
  well, until it is shown again. Visibility only applies to the session and is
  reset when the project changes.
- **Data integrity**: when the project is saved the CRC32 of the data file is
  computed; when it is loaded the code is verified again and a modified file is
  rejected.
- **Interface**: light/dark theme, compact style for notebooks, Italian /
  English / Portuguese languages. The selection — list row, list item, tab or
  active button — is always and only a frame, never a fill: this way the text
  stays readable in both themes.

## Data file format

The file (`.csv`, `.txt`, `.xlsx`, `.xls`) must be organized **one row per
sample**, with the **first row reserved for the headers**:

| Column | Content | Notes |
| --- | --- | --- |
| 1 | **Date** | see the accepted formats below |
| 2 | **Time** | optional if column 1 already contains date and time |
| 3 | **Sensor 1** | numeric value |
| 4 | **Sensor 2** | numeric value |
| *n* | ***Sensor n-2*** | it goes on forever |

The columns from the third on are all sensor series: there can be two or twenty
of them, with no limit. **The names of columns 3, 4, … become the sensor
names** and are used for the chart labels and for the 3D map, where every
sensor must be placed on the container. The two header cells of columns 1 and 2
are ignored.

Example with comma separator:

```csv
Date,Time,Sensor_1,Sensor_2
2026-06-01,08:00:00,18.5,19.2
2026-06-01,09:00:00,20.1,21.0
2026-06-01,10:00:00,22.4,22.8
```

### Field separator

Both the **comma `,`** and the **semicolon `;`** are accepted, detected
automatically from the first row of the file: nothing has to be specified. A
classic European file, with `;` as separator and decimal comma, is read without
conversions:

```csv
Date;Time;Sensor_1;Sensor_2
2026-06-01;08:00:00;18,5;19,2
2026-06-01;09:00:00;20,1;21,0
```

The detection counts the separators **outside** the quotes on the header row
only, so a text field containing a comma or a `;` cannot fool it. When the
counts are equal, the comma wins.

### Date and time formats

The first and second column are recombined into a single date and time value
and tried against a list of formats; the first one that matches wins.

With date and time separated in two columns:

| Example | Format |
| --- | --- |
| `2026-06-01` + `08:00:00` | `%Y-%m-%d %H:%M:%S` |
| `01/06/2026` + `08:00:00` | `%d/%m/%Y %H:%M:%S` |
| `2026-06-01` + `08:00` | `%Y-%m-%d %H:%M` |
| `01/06/2026` + `08:00` | `%d/%m/%Y %H:%M` |
| `2026/06/01` + `08:00:00` | `%Y/%m/%d %H:%M:%S` |
| `01-06-2026` + `08:00:00` | `%d-%m-%Y %H:%M:%S` |
| `2026-06-01` + `08:00:00.500` | `%Y-%m-%d %H:%M:%S%.f` (fractional) |
| `01/06/2026` + `08:00:00.500` | `%d/%m/%Y %H:%M:%S%.f` (fractional) |

With column 1 only (column 2 empty) it also accepts the date alone, as long as
it is in one of these formats, and the time is set to midnight:

`2026-06-01` · `01/06/2026` · `01-06-2026` · `2026/06/01`

### The time 24:00

The time `24:00:00` is valid and means the midnight that **closes** the day, as
in ISO 8601. It is interpreted as the `00:00:00` of the following day, so a log
that goes from `00:00:00` to `24:00:00` covers 24 full hours with 25 rows, and
the last sample does not end up overlapping the first one.

It works both in column 2 and when date and time are together in column 1
(`2026-06-01 24:00:00`).

### Example archive

`data/example_data.csv` is a ready-to-use example file: a whole day
(17/10/2026), one record per hour from `00:00:00` to `24:00:00`, 4 sensors with
random values between 2 and 8 °C. It uses the `;` separator and the decimal
comma, to test the automatic separator detection as well.

It is generated with a fixed seed (`genera_dati_fasulli.ps1`), so the content
never changes from one regeneration to the next.

### Numeric values

- Both the dot and the **decimal comma** are accepted: `18.5` and `18,5` read
  the same value.
- Empty or non-numeric cells are skipped for that sensor, without blocking the
  other series.
- Rows with an unrecognizable date/time are ignored completely.

### Limits to remember

- The first row is always interpreted as a header, so it must not be omitted
  and must not contain data.
- For Excel files the first sheet is used.
- The separator can be `,` or `;`, but **not the tab**: a TSV file has to be
  converted.

## Building

Requires [Rust](https://rustup.rs/) with the MSVC toolchain on Windows (or any
platform supported by `eframe`).

```sh
cargo build --release
```

The executable is in `target/release/thermite.exe`.

### Version resource on Windows

`build.rs` embeds `assets/thermite.ico` and a VERSIONINFO block
(FileDescription, ProductName, version, license) into the executable using
`rc.exe` of the Windows SDK. If `rc.exe` is not found the build goes on
normally, without just the version resource.

## Running

```sh
cargo run --release
```

The projects are saved in `advanced_projects.db` (SQLite) and the preferences
in `thermite_settings.txt`. Neither of them should be versioned.

When the database is missing it is created automatically, in the first folder
that can be written, searched in this order:

1. the working directory, but only if `advanced_projects.db` is already there
   (the behaviour of the previous versions);
2. the folder of the executable;
3. `%LOCALAPPDATA%\Thermite`.

The preferences file keeps being read from and written to the working
directory.

## Structure

| Path | Content |
| --- | --- |
| `src/main.rs` | entry point, command handling |
| `src/data.rs` | CSV/Excel parsing and time series |
| `src/crc32.rs` | CRC32 IEEE 802.3 of the data file |
| `src/models.rs` | models and SQLite database access |
| `src/theme.rs` | palette and styles of the two themes |
| `src/i18n.rs` | IT / EN / PT-BR translation tables |
| `src/scene3d.rs` | 3D geometry and projection of the scene |
| `src/icon.rs` | icon drawn procedurally |
| `src/views/` | tabs: Management, Search, Data, Chart, 3D Sensors, Settings |
| `src/export.rs` | PDF export of charts and map |

## Tests

```sh
cargo test
```

They cover parsing, 3D geometry, CRC32, database migration, PDF export and icon
drawing.

## License

GNU GENERAL PUBLIC LICENSE
Version 3, 29 June 2007
