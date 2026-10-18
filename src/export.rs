//! Esportazione del grafico termico in PDF (A4 orizzontale, grafica vettoriale).

use crate::data::{self, Bounds, Series};
use crate::i18n::{self, Language};
use crate::models::{Project, SceneDims, SensorPos};
use crate::scene3d;
use crate::views::{MapPreview, ViewState};
use printpdf::*;
use std::path::Path;

const PAGE_W: f32 = 297.0;
const PAGE_H: f32 = 210.0;

// Area del grafico (mm, origine in basso a sinistra).
pub(crate) const LEFT: f32 = 44.0;
pub(crate) const RIGHT: f32 = 283.0;
pub(crate) const BOTTOM: f32 = 30.0;
pub(crate) const TOP: f32 = 170.0;

/// Dimensioni della pagina A4 orizzontale in millimetri.
pub(crate) const PAGE_SIZE_MM: (f32, f32) = (PAGE_W, PAGE_H);

/// Genera il PDF del grafico termico e lo scrive su disco.
pub fn export_chart_pdf(
    path: &Path,
    project: &Project,
    series: &[Series],
    lang: Language,
) -> Result<(), String> {
    let bounds = data::bounds(series, project.min_temp, project.max_temp).ok_or_else(|| {
        i18n::t(lang, "Nessun dato numerico da rappresentare.").to_string()
    })?;

    let ops = build_page(project, series, bounds, lang);
    let mut doc = PdfDocument::new(&format!(
        "{} - {}",
        i18n::t(lang, "Grafico termico"),
        project.name
    ));
    let bytes = doc
        .with_pages(vec![PdfPage::new(Mm(PAGE_W), Mm(PAGE_H), ops)])
        .save(&PdfSaveOptions::default(), &mut Vec::new());

    std::fs::write(path, bytes)
        .map_err(|e| format!("{} {e}", i18n::t(lang, "Impossibile scrivere il file:")))
}

// ---------------------------------------------------------------------------
// Costruzione della pagina
// ---------------------------------------------------------------------------

fn build_page(project: &Project, series: &[Series], b: Bounds, lang: Language) -> Vec<Op> {
    let mut ops: Vec<Op> = Vec::new();

    let text_color = rgb(0x20, 0x20, 0x20);
    let muted = rgb(0x70, 0x70, 0x70);

    // --- Intestazione ---
    ops.extend(text(
        LEFT,
        199.0,
        i18n::t(lang, "Grafico termico"),
        17.0,
        true,
        text_color,
    ));
    ops.extend(text(
        LEFT,
        192.5,
        &project.name,
        12.0,
        false,
        rgb(0x21, 0x73, 0x46),
    ));

    // --- Riga informazioni ---
    let info = format!(
        "{}: {}    {}: {:.1} °C / {:.1} °C    {}: {}    {}: {}",
        i18n::t(lang, "Container"),
        if project.container_type.trim().is_empty() {
            "-"
        } else {
            project.container_type.as_str()
        },
        i18n::t(lang, "Soglie"),
        project.min_temp,
        project.max_temp,
        i18n::t(lang, "Sensori"),
        series.iter().filter(|s| !s.points.is_empty()).count(),
        i18n::t(lang, "Campioni"),
        data::total_points(series),
    );
    ops.extend(text(LEFT, 187.0, &info, 8.5, false, muted));

    let generated = format!(
        "{} {}",
        i18n::t(lang, "Generato il"),
        chrono::Local::now().format("%d/%m/%Y %H:%M")
    );
    let gw = text_w_mm(&generated, 8.5);
    ops.extend(text(RIGHT - gw, 187.0, &generated, 8.5, false, muted));

    // File dati di origine
    ops.extend(text(
        LEFT,
        182.5,
        &format!(
            "{} {}",
            i18n::t(lang, "File dati:"),
            shorten(&project.data_file_path, 120)
        ),
        8.0,
        false,
        muted,
    ));

    // --- Legenda ---
    ops.extend(legend_ops(series, 176.5));

    // --- Sfondo area grafico ---
    ops.push(Op::SetFillColor {
        col: rgb(0xFA, 0xFA, 0xFA).to_pdf(),
    });
    ops.push(Op::DrawPolygon {
        polygon: rect_polygon(LEFT, BOTTOM, RIGHT, TOP),
    });

    // --- Griglia e assi ---
    let y_step = nice_step(b.y_max - b.y_min, 6.0);
    let mut y_ticks: Vec<f64> = Vec::new();
    let mut y = (b.y_min / y_step).ceil() * y_step;
    while y <= b.y_max + 1e-9 {
        y_ticks.push(y);
        y += y_step;
    }

    for ty in &y_ticks {
        let py = map_y(*ty, b);
        ops.extend(line(
            vec![(LEFT, py), (RIGHT, py)],
            0.25,
            rgb(0xD9, 0xD9, 0xD9),
            false,
        ));
    }

    let x_ticks = x_tick_values(b, 6);
    for tx in &x_ticks {
        let px = map_x(*tx, b);
        ops.extend(line(
            vec![(px, BOTTOM), (px, TOP)],
            0.25,
            rgb(0xE4, 0xE4, 0xE4),
            false,
        ));
    }

    // --- Soglie di progetto ---
    let max_y = map_y(project.max_temp as f64, b);
    ops.extend(line(
        vec![(LEFT, max_y), (RIGHT, max_y)],
        0.7,
        rgb(0xD6, 0x27, 0x28),
        true,
    ));
    let max_label = format!("{} {:.1} °C", i18n::t(lang, "Max"), project.max_temp);
    ops.extend(text(
        RIGHT - text_w_mm(&max_label, 7.5) - 1.0,
        max_y + 1.2,
        &max_label,
        7.5,
        true,
        rgb(0xD6, 0x27, 0x28),
    ));

    let min_y = map_y(project.min_temp as f64, b);
    ops.extend(line(
        vec![(LEFT, min_y), (RIGHT, min_y)],
        0.7,
        rgb(0x1F, 0x77, 0xB4),
        true,
    ));
    let min_label = format!("{} {:.1} °C", i18n::t(lang, "Min"), project.min_temp);
    ops.extend(text(
        RIGHT - text_w_mm(&min_label, 7.5) - 1.0,
        min_y + 1.2,
        &min_label,
        7.5,
        true,
        rgb(0x1F, 0x77, 0xB4),
    ));

    // --- Serie dei sensori ---
    for (i, s) in series.iter().enumerate() {
        if s.points.len() < 1 {
            continue;
        }
        let color = series_color(i);
        let pts: Vec<(f32, f32)> = s
            .points
            .iter()
            .map(|p| (map_x(p[0], b), map_y(p[1], b)))
            .collect();

        if pts.len() > 1 {
            ops.extend(line(pts.clone(), 1.0, color, false));
        }
        // Marcatori solo se i punti non sono troppi (leggibilità)
        if pts.len() <= 90 {
            for (px, py) in &pts {
                ops.push(Op::SetFillColor { col: color.to_pdf() });
                ops.push(Op::DrawPolygon {
                    polygon: rect_polygon(px - 0.55, py - 0.55, px + 0.55, py + 0.55),
                });
            }
        }
    }

    // --- Bordo area grafico ---
    ops.push(Op::SetOutlineColor {
        col: rgb(0x9A, 0x9A, 0x9A).to_pdf(),
    });
    ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
    ops.push(Op::DrawPolygon {
        polygon: Polygon {
            rings: vec![PolygonRing {
                points: vec![
                    lp(LEFT, BOTTOM),
                    lp(RIGHT, BOTTOM),
                    lp(RIGHT, TOP),
                    lp(LEFT, TOP),
                ],
            }],
            mode: PaintMode::Stroke,
            winding_order: WindingOrder::NonZero,
        },
    });

    // --- Etichette assi ---
    for ty in &y_ticks {
        let py = map_y(*ty, b);
        let label = format!("{:.1}", ty);
        let w = text_w_mm(&label, 7.5);
        ops.extend(text(LEFT - 3.0 - w, py - 1.1, &label, 7.5, false, muted));
    }
    ops.extend(text(
        LEFT - 3.0 - text_w_mm("°C", 7.5),
        TOP + 1.0,
        "°C",
        7.5,
        false,
        muted,
    ));

    for tx in &x_ticks {
        let px = map_x(*tx, b);
        let label = data::format_timestamp(*tx);
        // due righe: data e ora, per non sovrapporre le etichette
        let (d, t) = label.split_once(' ').unwrap_or((label.as_str(), ""));
        let w = text_w_mm(&d, 7.0).max(text_w_mm(t, 7.0));
        ops.extend(text(px - w / 2.0, 24.5, t, 7.0, false, muted));
        ops.extend(text(px - text_w_mm(&d, 7.0) / 2.0, 20.5, &d, 7.0, false, muted));
    }
    ops.extend(text(
        LEFT,
        24.5,
        i18n::t(lang, "Data e ora"),
        7.5,
        true,
        muted,
    ));

    // --- Piè di pagina ---
    ops.extend(text(
        LEFT,
        11.0,
        i18n::t(lang, "Thermite · Mappatura Termica di Trasporto"),
        7.0,
        false,
        muted,
    ));

    ops
}

// ---------------------------------------------------------------------------
// Legenda
// ---------------------------------------------------------------------------

fn legend_ops(series: &[Series], y_top: f32) -> Vec<Op> {
    let mut ops = Vec::new();
    let mut x = LEFT;
    let mut row = 0usize;
    let label_size = 8.0;

    for (i, s) in series.iter().enumerate() {
        if s.points.is_empty() {
            continue;
        }
        let item_w = 5.5 + text_w_mm(&s.name, label_size) + 6.0;
        if x + item_w > RIGHT && row == 0 {
            row = 1;
            x = LEFT;
        }
        let y = y_top - row as f32 * 6.0;

        let color = series_color(i);
        ops.extend(line(vec![(x, y), (x + 5.0, y)], 1.6, color, false));
        ops.push(Op::SetFillColor { col: color.to_pdf() });
        ops.push(Op::DrawPolygon {
            polygon: rect_polygon(x + 2.0, y - 0.6, x + 3.2, y + 0.6),
        });
        ops.extend(text(
            x + 6.5,
            y - 1.1,
            &s.name,
            label_size,
            false,
            rgb(0x30, 0x30, 0x30),
        ));
        x += item_w;
    }
    ops
}

// ---------------------------------------------------------------------------
// Utilità geometriche
// ---------------------------------------------------------------------------

pub(crate) fn map_x(x: f64, b: Bounds) -> f32 {
    let t = ((x - b.x_min) / (b.x_max - b.x_min)) as f32;
    LEFT + t * (RIGHT - LEFT)
}

pub(crate) fn map_y(y: f64, b: Bounds) -> f32 {
    let t = ((y - b.y_min) / (b.y_max - b.y_min)) as f32;
    BOTTOM + t * (TOP - BOTTOM)
}

pub(crate) fn x_tick_values(b: Bounds, count: usize) -> Vec<f64> {
    let n = count.max(2);
    (0..n)
        .map(|i| b.x_min + (b.x_max - b.x_min) * i as f64 / (n - 1) as f64)
        .collect()
}

/// Passo "gradevole" per l'asse Y (1, 2, 5 × 10^n).
pub(crate) fn nice_step(range: f64, target: f64) -> f64 {
    if range <= 0.0 {
        return 1.0;
    }
    let raw = range / target;
    let mag = 10f64.powf(raw.log10().floor());
    let norm = raw / mag;
    let step = if norm <= 1.0 {
        1.0
    } else if norm <= 2.0 {
        2.0
    } else if norm <= 5.0 {
        5.0
    } else {
        10.0
    };
    (step * mag).max(0.1)
}

fn rect_polygon(x0: f32, y0: f32, x1: f32, y1: f32) -> Polygon {
    Polygon {
        rings: vec![PolygonRing {
            points: vec![lp(x0, y0), lp(x1, y0), lp(x1, y1), lp(x0, y1)],
        }],
        mode: PaintMode::Fill,
        winding_order: WindingOrder::NonZero,
    }
}

fn lp(x: f32, y: f32) -> LinePoint {
    LinePoint {
        p: Point::new(Mm(x), Mm(y)),
        bezier: false,
    }
}

fn point(x: f32, y: f32) -> Point {
    Point::new(Mm(x), Mm(y))
}

// ---------------------------------------------------------------------------
// Utilità di disegno text / linee
// ---------------------------------------------------------------------------

fn line(points: Vec<(f32, f32)>, width: f32, color: Rgbc, dashed: bool) -> Vec<Op> {
    let mut ops = Vec::new();
    ops.push(Op::SetOutlineColor { col: color.to_pdf() });
    ops.push(Op::SetOutlineThickness { pt: Pt(width) });
    ops.push(Op::SetLineCapStyle {
        cap: LineCapStyle::Round,
    });
    ops.push(Op::SetLineJoinStyle {
        join: LineJoinStyle::Round,
    });
    if dashed {
        ops.push(Op::SetLineDashPattern {
            dash: LineDashPattern::new(0.0, &[3.0, 2.5]),
        });
    } else {
        ops.push(Op::SetLineDashPattern {
            dash: LineDashPattern::new(0.0, &[]),
        });
    }
    ops.push(Op::DrawLine {
        line: Line {
            points: points.into_iter().map(|(x, y)| lp(x, y)).collect(),
            is_closed: false,
        },
    });
    ops
}

fn text(x: f32, y: f32, content: &str, size: f32, bold: bool, color: Rgbc) -> Vec<Op> {
    vec![
        Op::StartTextSection,
        Op::SetTextCursor { pos: point(x, y) },
        Op::SetFont {
            font: PdfFontHandle::Builtin(if bold {
                BuiltinFont::HelveticaBold
            } else {
                BuiltinFont::Helvetica
            }),
            size: Pt(size),
        },
        Op::SetLineHeight { lh: Pt(size) },
        Op::SetFillColor { col: color.to_pdf() },
        Op::ShowText {
            items: vec![TextItem::Text(content.to_string())],
        },
        Op::EndTextSection,
    ]
}

/// Larghezza stimata del testo in mm (Helvetica, larghezza media ≈ 0,5 em).
fn text_w_mm(s: &str, size_pt: f32) -> f32 {
    s.chars().count() as f32 * size_pt * 0.5 * 0.352_778
}

/// Colore RGB "Copy", convertito in [`printpdf::Color`] al momento del disegno.
#[derive(Clone, Copy)]
struct Rgbc(u8, u8, u8);

impl Rgbc {
    fn to_pdf(self) -> Color {
        Color::Rgb(Rgb {
            r: self.0 as f32 / 255.0,
            g: self.1 as f32 / 255.0,
            b: self.2 as f32 / 255.0,
            icc_profile: None,
        })
    }
}

fn rgb(r: u8, g: u8, b: u8) -> Rgbc {
    Rgbc(r, g, b)
}

fn series_color(i: usize) -> Rgbc {
    let c = crate::theme::series_color(i);
    Rgbc(c.r(), c.g(), c.b())
}

fn shorten(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

// ---------------------------------------------------------------------------
// Comando dell'interfaccia: salvataggio su file
// ---------------------------------------------------------------------------

impl ViewState {
    /// Chiede il percorso di destinazione e scrive il PDF del grafico indicato.
    pub fn write_pdf_with_dialog(&mut self, project: &Project, series: &[Series]) {
        let lang = self.lang;
        let default_name = format!("{}.pdf", safe_file_name(&project.name));
        let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::t(lang, "Salva il grafico termico in PDF"))
            .set_file_name(&default_name)
            .add_filter(i18n::t(lang, "Documento PDF"), &["pdf"])
            .save_file()
        else {
            self.set_status(i18n::t(lang, "Esportazione PDF annullata."), false);
            return;
        };

        let path = if path.extension().is_some() {
            path
        } else {
            path.with_extension("pdf")
        };

        match export_chart_pdf(&path, project, series, lang) {
            Ok(()) => self.set_status(
                format!(
                    "{} {}",
                    i18n::t(lang, "Grafico esportato in PDF:"),
                    path.display()
                ),
                false,
            ),
            Err(e) => self.set_status(
                format!("{} {e}", i18n::t(lang, "Errore esportazione PDF:")),
                true,
            ),
        }
    }
}

fn safe_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        "grafico_termico".to_string()
    } else {
        cleaned.to_string()
    }
}

// ---------------------------------------------------------------------------
// Esportazione della mappa 3D dei sensori
// ---------------------------------------------------------------------------

/// Genera il PDF della mappa 3D dei sensori e lo scrive su disco.
pub fn export_map_pdf(
    path: &Path,
    project: &Project,
    dims: SceneDims,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    positions: &[SensorPos],
    stats: &[(String, f32, f32)],
    lang: Language,
) -> Result<(), String> {
    let ops = build_map_page(project, dims, yaw, pitch, zoom, positions, stats, lang);
    let mut doc = PdfDocument::new(&format!(
        "{} - {}",
        i18n::t(lang, "Mappa sensori 3D"),
        project.name
    ));
    let bytes = doc
        .with_pages(vec![PdfPage::new(Mm(PAGE_W), Mm(PAGE_H), ops)])
        .save(&PdfSaveOptions::default(), &mut Vec::new());

    std::fs::write(path, bytes)
        .map_err(|e| format!("{} {e}", i18n::t(lang, "Impossibile scrivere il file:")))
}

#[allow(clippy::too_many_arguments)]
fn build_map_page(
    project: &Project,
    dims: SceneDims,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    positions: &[SensorPos],
    stats: &[(String, f32, f32)],
    lang: Language,
) -> Vec<Op> {
    let mut ops: Vec<Op> = Vec::new();
    let text_color = rgb(0x20, 0x20, 0x20);
    let muted = rgb(0x70, 0x70, 0x70);

    // --- Intestazione ---
    ops.extend(text(
        LEFT,
        199.0,
        i18n::t(lang, "Mappa sensori 3D"),
        17.0,
        true,
        text_color,
    ));
    ops.extend(text(
        LEFT,
        192.5,
        &project.name,
        12.0,
        false,
        rgb(0x21, 0x73, 0x46),
    ));

    let info = format!(
        "{}: {}    {}: {:.1} × {:.1} × {:.1} m    {}: {}",
        i18n::t(lang, "Container"),
        if project.container_type.trim().is_empty() {
            "-"
        } else {
            project.container_type.as_str()
        },
        i18n::t(lang, "Dimensioni"),
        dims.w,
        dims.h,
        dims.d,
        i18n::t(lang, "Sensori"),
        positions.len(),
    );
    ops.extend(text(LEFT, 187.0, &info, 8.5, false, muted));

    let generated = format!(
        "{} {}",
        i18n::t(lang, "Generato il"),
        chrono::Local::now().format("%d/%m/%Y %H:%M")
    );
    let gw = text_w_mm(&generated, 8.5);
    ops.extend(text(RIGHT - gw, 187.0, &generated, 8.5, false, muted));

    // --- Sfondo area ---
    ops.push(Op::SetFillColor {
        col: rgb(0xFA, 0xFA, 0xFA).to_pdf(),
    });
    ops.push(Op::DrawPolygon {
        polygon: rect_polygon(LEFT, BOTTOM, RIGHT, TOP),
    });

    // --- Scena 3D ---
    let cx = (LEFT + RIGHT) * 0.5;
    let cy = (BOTTOM + TOP) * 0.5;
    let viewport = (RIGHT - LEFT).min(TOP - BOTTOM);
    let cam = scene3d::Camera { yaw, pitch, zoom };
    let proj = scene3d::Projector::new(cx, cy, viewport, &cam, dims);
    let flip = |y: f32| 2.0 * cy - y;

    for q in scene3d::face_quads(&proj, dims) {
        let pts: Vec<(f32, f32)> = q.pts.iter().map(|(x, y)| (*x, flip(*y))).collect();
        ops.push(Op::SetFillColor {
            col: shade_pdf(0xD9, 0xDD, 0xE1, q.shade).to_pdf(),
        });
        ops.push(Op::DrawPolygon {
            polygon: poly(pts.clone(), PaintMode::Fill),
        });
        ops.push(Op::SetOutlineColor {
            col: rgb(0x60, 0x60, 0x60).to_pdf(),
        });
        ops.push(Op::SetOutlineThickness { pt: Pt(0.6) });
        ops.push(Op::SetLineJoinStyle {
            join: LineJoinStyle::Round,
        });
        ops.push(Op::DrawPolygon {
            polygon: poly(pts, PaintMode::Stroke),
        });
    }

    // --- Sensori: punto, nome, temperatura minima / massima ---
    for (i, p) in positions.iter().enumerate() {
        let (x, y) = proj.project(scene3d::sensor_point(p, dims));
        let (px, py) = (x, flip(y));
        let color = series_color(i);

        ops.push(Op::SetFillColor { col: color.to_pdf() });
        ops.push(Op::DrawPolygon {
            polygon: poly(circle_points(px, py, 1.5, 18), PaintMode::Fill),
        });
        ops.push(Op::SetOutlineColor {
            col: rgb(0xFF, 0xFF, 0xFF).to_pdf(),
        });
        ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
        ops.push(Op::DrawPolygon {
            polygon: poly(circle_points(px, py, 1.5, 18), PaintMode::Stroke),
        });

        let (lo, hi) = stats
            .iter()
            .find(|(n, _, _)| *n == p.name)
            .map(|(_, a, b)| (*a, *b))
            .unwrap_or((f32::NAN, f32::NAN));
        ops.extend(text(px + 2.2, py + 1.6, &p.name, 8.0, true, text_color));
        let temps = if lo.is_finite() {
            format!("{lo:.1} / {hi:.1} °C")
        } else {
            "-".to_string()
        };
        ops.extend(text(px + 2.2, py - 1.4, &temps, 7.0, false, muted));
    }

    // --- Bordo area ---
    ops.push(Op::SetOutlineColor {
        col: rgb(0x9A, 0x9A, 0x9A).to_pdf(),
    });
    ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
    ops.push(Op::SetLineDashPattern {
        dash: LineDashPattern::new(0.0, &[]),
    });
    ops.push(Op::DrawPolygon {
        polygon: poly(
            vec![(LEFT, BOTTOM), (RIGHT, BOTTOM), (RIGHT, TOP), (LEFT, TOP)],
            PaintMode::Stroke,
        ),
    });

    // --- Piè di pagina ---
    ops.extend(text(
        LEFT,
        11.0,
        i18n::t(lang, "Thermite · Mappatura Termica di Trasporto"),
        7.0,
        false,
        muted,
    ));

    ops
}

/// Poligono a partire da una lista di punti (mm), con la modalità indicata.
fn poly(points: Vec<(f32, f32)>, mode: PaintMode) -> Polygon {
    Polygon {
        rings: vec![PolygonRing {
            points: points.into_iter().map(|(x, y)| lp(x, y)).collect(),
        }],
        mode,
        winding_order: WindingOrder::NonZero,
    }
}

/// Approssimazione poligonale di un cerchio (mm).
fn circle_points(cx: f32, cy: f32, r: f32, n: usize) -> Vec<(f32, f32)> {
    (0..n)
        .map(|i| {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// Colore RGB moltiplicato per un fattore di luce.
fn shade_pdf(r: u8, g: u8, b: u8, k: f32) -> Rgbc {
    let f = |v: u8| (v as f32 * k).clamp(0.0, 255.0) as u8;
    Rgbc(f(r), f(g), f(b))
}

impl ViewState {
    /// Chiede il percorso di destinazione e scrive il PDF della mappa indicata.
    pub fn write_map_pdf_with_dialog(&mut self, preview: &MapPreview) {
        let lang = self.lang;
        let default_name = format!("{}_mappa.pdf", safe_file_name(&preview.project.name));
        let Some(path) = rfd::FileDialog::new()
            .set_title(i18n::t(lang, "Salva la mappa dei sensori in PDF"))
            .set_file_name(&default_name)
            .add_filter(i18n::t(lang, "Documento PDF"), &["pdf"])
            .save_file()
        else {
            self.set_status(i18n::t(lang, "Esportazione PDF annullata."), false);
            return;
        };

        let path = if path.extension().is_some() {
            path
        } else {
            path.with_extension("pdf")
        };

        match export_map_pdf(
            &path,
            &preview.project,
            preview.dims,
            preview.yaw,
            preview.pitch,
            preview.zoom,
            &preview.positions,
            &preview.stats,
            lang,
        ) {
            Ok(()) => self.set_status(
                format!(
                    "{} {}",
                    i18n::t(lang, "Mappa esportata in PDF:"),
                    path.display()
                ),
                false,
            ),
            Err(e) => self.set_status(
                format!("{} {e}", i18n::t(lang, "Errore esportazione PDF:")),
                true,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project() -> Project {
        Project {
            id: 1,
            name: "Camera Test".to_string(),
            container_type: "Container 20piedi".to_string(),
            min_temp: -5.0,
            max_temp: 25.0,
            data_file_path: "data/test.csv".to_string(),
            data_crc32: None,
        }
    }

    fn series() -> Vec<Series> {
        let mut s = Series {
            name: "Sensore 1".to_string(),
            points: Vec::new(),
            stamps: Vec::new(),
        };
        for i in 0..50 {
            s.points.push([1_700_000_000.0 + i as f64 * 600.0, (i as f64 * 0.7).sin() * 8.0]);
            s.stamps.push(("01/01/2024".to_string(), "12:00".to_string()));
        }
        let mut s2 = s.clone();
        s2.name = "Sensore 2".to_string();
        vec![s, s2]
    }

    #[test]
    fn genera_pdf_valido() {
        let dir = std::env::temp_dir().join("thermite_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("grafico.pdf");

        export_chart_pdf(&path, &project(), &series(), Language::It).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() > 1500, "PDF troppo piccolo: {}", bytes.len());
        assert_eq!(&bytes[0..4], b"%PDF", "intestazione PDF mancante");
    }

    #[test]
    fn genera_pdf_mappa_valido() {
        let dir = std::env::temp_dir().join("thermite_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("mappa.pdf");

        let positions = vec![
            SensorPos {
                name: "Sensore 1".to_string(),
                face: 4,
                u: 0.25,
                v: 0.5,
                inside: false,
            },
            // Lato interno: il test copre anche la legenda che il PDF deve
            // mostrare quando uno o più sensori sono dentro il container.
            SensorPos {
                name: "Sensore 2".to_string(),
                face: 2,
                u: 0.7,
                v: 0.3,
                inside: true,
            },
        ];
        let stats = vec![
            ("Sensore 1".to_string(), -3.2_f32, 7.8_f32),
            ("Sensore 2".to_string(), -1.0_f32, 9.5_f32),
        ];

        export_map_pdf(
            &path,
            &project(),
            SceneDims::default(),
            0.6,
            0.4,
            1.0,
            &positions,
            &stats,
            Language::It,
        )
        .unwrap();

        let bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() > 1500, "PDF troppo piccolo: {}", bytes.len());
        assert_eq!(&bytes[0..4], b"%PDF", "intestazione PDF mancante");
    }
}
