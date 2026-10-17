//! Anteprima del PDF: mostra a video la pagina A4 orizzontale esattamente come
//! verrà generata, con i pulsanti per salvarla o chiudere.

use crate::data::{self, Series};
use crate::export;
use crate::i18n::{self, Language};
use crate::models::Project;
use crate::theme::{self, ButtonKind, Icon};
use crate::views::ViewState;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Rect, Stroke, StrokeKind, Vec2,
};

/// Dati necessari a disegnare l'anteprima (restano in memoria finché è aperta).
pub struct PdfPreview {
    pub project: Project,
    pub series: Vec<Series>,
}

// Colori "carta": l'anteprima rappresenta il foglio stampato, quindi non
// segue il tema chiaro/scuro dell'applicazione.
const PAPER: Color32 = Color32::WHITE;
const INK: Color32 = Color32::from_rgb(0x20, 0x20, 0x20);
const INK_MUTED: Color32 = Color32::from_rgb(0x70, 0x70, 0x70);
const GRID: Color32 = Color32::from_rgb(0xE0, 0xE0, 0xE0);
const AREA_BG: Color32 = Color32::from_rgb(0xFA, 0xFA, 0xFA);
const AXIS: Color32 = Color32::from_rgb(0x9A, 0x9A, 0x9A);
const ACCENT: Color32 = Color32::from_rgb(0x21, 0x73, 0x46);

impl ViewState {
    /// Apre la finestra di anteprima per il progetto attivo.
    pub fn open_pdf_preview(&mut self) {
        let lang = self.lang;
        let Some(project) = self.active_project.clone() else {
            self.set_status(
                i18n::t(
                    lang,
                    "Nessun progetto attivo: apri prima i dati di un progetto dalla scheda Ricerca.",
                ),
                true,
            );
            return;
        };

        if self.csv_rows.is_empty() {
            self.load_data_file();
        }
        let series = data::parse_series(&self.csv_headers, &self.csv_rows);
        if data::total_points(&series) == 0 {
            self.set_status(
                i18n::t(
                    lang,
                    "Nessun dato numerico da esportare: controlla il file dati del progetto.",
                ),
                true,
            );
            return;
        }

        self.pdf_preview = Some(PdfPreview { project, series });
        self.set_status(
            i18n::t(lang, "Anteprima PDF pronta: controlla e salva."),
            false,
        );
    }

    /// Disegna la finestra di anteprima (se aperta).
    pub fn show_pdf_preview_window(&mut self, ctx: &egui::Context) {
        let Some(preview) = self.pdf_preview.take() else {
            return;
        };
        let lang = self.lang;

        let mut open = true;
        let mut do_save = false;
        let mut do_close = false;

        egui::Window::new(i18n::t(lang, "Anteprima PDF — Grafico termico"))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_size(Vec2::new(980.0, 730.0))
            .min_width(640.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if theme::action_button(
                        ui,
                        Icon::Pdf,
                        i18n::t(lang, "Salva PDF…"),
                        ButtonKind::Primary,
                    )
                    .on_hover_text(i18n::t(
                        lang,
                        "Apri la finestra di salvataggio e scrivi il file",
                    ))
                    .clicked()
                    {
                        do_save = true;
                    }
                    if theme::action_button(
                        ui,
                        Icon::Close,
                        i18n::t(lang, "Chiudi"),
                        ButtonKind::Neutral,
                    )
                    .clicked()
                    {
                        do_close = true;
                    }
                    ui.separator();
                    theme::hint(
                        ui,
                        i18n::t(lang, "Così apparirà la pagina A4 orizzontale salvata."),
                    );
                });
                ui.add_space(8.0);

                egui::ScrollArea::both().show(ui, |ui| {
                    let page_w = ui.available_width().clamp(240.0, 1000.0);
                    let (pw_mm, ph_mm) = export::PAGE_SIZE_MM;
                    let page_h = page_w * (ph_mm / pw_mm);
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(page_w, page_h), egui::Sense::hover());
                    draw_page(ui.painter(), rect, &preview.project, &preview.series, lang);
                });
            });

        if do_save {
            self.write_pdf_with_dialog(&preview.project, &preview.series);
            self.pdf_preview = None;
            return;
        }
        if do_close || !open {
            self.set_status(i18n::t(lang, "Anteprima PDF chiusa."), false);
            return;
        }
        // La finestra resta aperta: rimetti i dati per il prossimo frame.
        self.pdf_preview = Some(preview);
    }
}

/// Disegna la pagina completa sul rettangolo indicato, replicando il layout del PDF.
fn draw_page(
    painter: &egui::Painter,
    page: Rect,
    project: &Project,
    series: &[Series],
    lang: Language,
) {
    let (pw_mm, ph_mm) = export::PAGE_SIZE_MM;
    let sx = page.width() / pw_mm;

    // Da millimetri del PDF (origine in basso a sinistra) a punti schermo.
    let mm = |x: f32, y: f32| {
        egui::pos2(
            page.left() + x * sx,
            page.top() + (ph_mm - y) * (page.height() / ph_mm),
        )
    };
    // Dimensione carattere: da punti tipografici a pixel schermo.
    let fs = |pt: f32| (pt * 0.352_778 * sx).max(4.5);

    // Foglio
    painter.rect_filled(page, CornerRadius::ZERO, PAPER);
    painter.rect_stroke(
        page,
        CornerRadius::ZERO,
        Stroke::new(1.0_f32, Color32::from_rgb(0xC8, 0xC8, 0xC8)),
        StrokeKind::Inside,
    );

    let Some(b) = data::bounds(series, project.min_temp, project.max_temp) else {
        painter.text(
            page.center(),
            Align2::CENTER_CENTER,
            i18n::t(lang, "Nessun dato da rappresentare"),
            FontId::proportional(fs(12.0)),
            Color32::from_rgb(0xD6, 0x27, 0x28),
        );
        return;
    };

    // --- Intestazione ---
    painter.text(
        mm(export::LEFT, 199.0),
        Align2::LEFT_BOTTOM,
        i18n::t(lang, "Grafico termico"),
        FontId::proportional(fs(17.0)),
        INK,
    );
    painter.text(
        mm(export::LEFT, 192.5),
        Align2::LEFT_BOTTOM,
        &project.name,
        FontId::proportional(fs(12.0)),
        ACCENT,
    );

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
    painter.text(
        mm(export::LEFT, 187.0),
        Align2::LEFT_BOTTOM,
        info,
        FontId::proportional(fs(8.5)),
        INK_MUTED,
    );
    painter.text(
        mm(export::RIGHT, 187.0),
        Align2::RIGHT_BOTTOM,
        format!(
            "{} {}",
            i18n::t(lang, "Generato il"),
            chrono::Local::now().format("%d/%m/%Y %H:%M")
        ),
        FontId::proportional(fs(8.5)),
        INK_MUTED,
    );
    painter.text(
        mm(export::LEFT, 182.5),
        Align2::LEFT_BOTTOM,
        format!("{} {}", i18n::t(lang, "File dati:"), project.data_file_path),
        FontId::proportional(fs(8.0)),
        INK_MUTED,
    );

    // --- Legenda (fino a due righe) ---
    let mut lx = export::LEFT;
    let mut row = 0usize;
    for (i, s) in series.iter().enumerate() {
        if s.points.is_empty() {
            continue;
        }
        let galley = painter.layout_no_wrap(
            s.name.clone(),
            FontId::proportional(fs(8.0)),
            INK_MUTED,
        );
        let name_mm = galley.size().x / sx;
        let item_w = 5.5 + name_mm + 6.0;
        if lx + item_w > export::RIGHT && row == 0 {
            row = 1;
            lx = export::LEFT;
        }
        let ly = 176.5 - row as f32 * 6.0;
        let color = theme::series_color(i);
        painter.line_segment(
            [mm(lx, ly), mm(lx + 5.0, ly)],
            Stroke::new(1.6_f32, color),
        );
        painter.rect_filled(
            Rect::from_two_pos(mm(lx + 2.0, ly - 0.6), mm(lx + 3.2, ly + 0.6)),
            CornerRadius::ZERO,
            color,
        );
        painter.text(
            mm(lx + 6.5, ly - 1.1),
            Align2::LEFT_BOTTOM,
            &s.name,
            FontId::proportional(fs(8.0)),
            INK_MUTED,
        );
        lx += item_w;
    }

    // --- Area del grafico ---
    let area = Rect::from_two_pos(
        mm(export::LEFT, export::BOTTOM),
        mm(export::RIGHT, export::TOP),
    );
    painter.rect_filled(area, CornerRadius::ZERO, AREA_BG);

    // Griglia orizzontale + etichette asse Y
    let step = export::nice_step(b.y_max - b.y_min, 6.0);
    let mut ty = (b.y_min / step).ceil() * step;
    let mut guard = 0;
    while ty <= b.y_max + 1e-9 && guard < 200 {
        let y = export::map_y(ty, b);
        painter.line_segment(
            [mm(export::LEFT, y), mm(export::RIGHT, y)],
            Stroke::new(0.6_f32, GRID),
        );
        painter.text(
            mm(export::LEFT - 3.0, y),
            Align2::RIGHT_CENTER,
            format!("{ty:.1}"),
            FontId::proportional(fs(7.5)),
            INK_MUTED,
        );
        ty += step;
        guard += 1;
    }
    painter.text(
        mm(export::LEFT - 3.0, export::TOP + 1.0),
        Align2::RIGHT_BOTTOM,
        "°C",
        FontId::proportional(fs(7.5)),
        INK_MUTED,
    );

    // Griglia verticale + etichette asse X
    let x_ticks = export::x_tick_values(b, 6);
    for tx in &x_ticks {
        let x = export::map_x(*tx, b);
        painter.line_segment(
            [mm(x, export::BOTTOM), mm(x, export::TOP)],
            Stroke::new(0.6_f32, Color32::from_rgb(0xE6, 0xE6, 0xE6)),
        );

        let label = data::format_timestamp(*tx);
        let (d, t) = label.split_once(' ').unwrap_or((label.as_str(), ""));
        painter.text(
            mm(x, 26.0),
            Align2::CENTER_BOTTOM,
            t,
            FontId::proportional(fs(7.0)),
            INK_MUTED,
        );
        painter.text(
            mm(x, 22.0),
            Align2::CENTER_BOTTOM,
            d,
            FontId::proportional(fs(7.0)),
            INK_MUTED,
        );
    }
    painter.text(
        mm(export::LEFT, 26.0),
        Align2::LEFT_BOTTOM,
        i18n::t(lang, "Data e ora"),
        FontId::proportional(fs(7.5)),
        INK_MUTED,
    );

    // --- Soglie (tratteggiate) ---
    let dash = 2.4 * sx;
    let gap = 1.9 * sx;
    let max_y = export::map_y(project.max_temp as f64, b);
    painter.extend(egui::Shape::dashed_line(
        &[mm(export::LEFT, max_y), mm(export::RIGHT, max_y)],
        Stroke::new(1.3_f32, theme::series_color(1)),
        dash,
        gap,
    ));
    painter.text(
        mm(export::RIGHT, max_y + 0.6),
        Align2::RIGHT_BOTTOM,
        format!("{} {:.1} °C", i18n::t(lang, "Max"), project.max_temp),
        FontId::proportional(fs(7.5)),
        theme::series_color(1),
    );

    let min_y = export::map_y(project.min_temp as f64, b);
    painter.extend(egui::Shape::dashed_line(
        &[mm(export::LEFT, min_y), mm(export::RIGHT, min_y)],
        Stroke::new(1.3_f32, theme::series_color(0)),
        dash,
        gap,
    ));
    painter.text(
        mm(export::RIGHT, min_y + 0.6),
        Align2::RIGHT_BOTTOM,
        format!("{} {:.1} °C", i18n::t(lang, "Min"), project.min_temp),
        FontId::proportional(fs(7.5)),
        theme::series_color(0),
    );

    // --- Serie ---
    for (i, s) in series.iter().enumerate() {
        if s.points.is_empty() {
            continue;
        }
        let color = theme::series_color(i);
        let pts: Vec<egui::Pos2> = s
            .points
            .iter()
            .map(|p| mm(export::map_x(p[0], b), export::map_y(p[1], b)))
            .collect();

        if pts.len() > 1 {
            painter.add(egui::Shape::line(pts.clone(), Stroke::new(1.5_f32, color)));
        }
        if pts.len() <= 90 {
            let m = 0.6 * sx;
            for p in &pts {
                painter.rect_filled(
                    Rect::from_min_max(egui::pos2(p.x - m, p.y - m), egui::pos2(p.x + m, p.y + m)),
                    CornerRadius::ZERO,
                    color,
                );
            }
        }
    }

    // --- Bordo area ---
    painter.rect_stroke(area, CornerRadius::ZERO, Stroke::new(1.0_f32, AXIS), StrokeKind::Inside);

    // --- Piè di pagina ---
    painter.text(
        mm(export::LEFT, 11.0),
        Align2::LEFT_BOTTOM,
        i18n::t(lang, "Thermite · Mappatura Termica di Trasporto"),
        FontId::proportional(fs(7.0)),
        INK_MUTED,
    );
}
