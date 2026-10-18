//! Graphical theme in "ArtCraft / GridCraft" style (Excel-like look) and small
//! hand-drawn vector icons, so the app does not depend on icon fonts.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Rect, RichText, Stroke, Vec2,
};

// ---------------------------------------------------------------------------
// Palette
// ---------------------------------------------------------------------------

/// "Excel" green used as the accent in the light theme.
pub const ACCENT_LIGHT: Color32 = Color32::from_rgb(0x21, 0x73, 0x46);
/// Brighter green, readable on the dark theme.
pub const ACCENT_DARK: Color32 = Color32::from_rgb(0x35, 0xB1, 0x7A);

pub const DANGER: Color32 = Color32::from_rgb(0xC0, 0x39, 0x2B);
pub const WARNING: Color32 = Color32::from_rgb(0xE8, 0x8A, 0x00);
pub const OK: Color32 = Color32::from_rgb(0x2E, 0x8B, 0x57);

pub fn accent(dark: bool) -> Color32 {
    if dark {
        ACCENT_DARK
    } else {
        ACCENT_LIGHT
    }
}

pub fn panel_bg(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0x1E, 0x1E, 0x1E)
    } else {
        Color32::from_rgb(0xF3, 0xF3, 0xF3)
    }
}

pub fn ribbon_bg(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0x2B, 0x2B, 0x2B)
    } else {
        Color32::WHITE
    }
}

pub fn card_bg(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0x26, 0x26, 0x27)
    } else {
        Color32::from_rgb(0xFB, 0xFB, 0xFB)
    }
}

pub fn border(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0x3C, 0x3C, 0x3C)
    } else {
        Color32::from_rgb(0xD6, 0xD6, 0xD6)
    }
}

pub fn muted(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0xA8, 0xA8, 0xA8)
    } else {
        Color32::from_rgb(0x5F, 0x63, 0x68)
    }
}

/// Foreground text color: full black in the light theme, white in the dark one.
pub fn text_strong(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0xF2, 0xF2, 0xF2)
    } else {
        Color32::from_rgb(0x00, 0x00, 0x00)
    }
}

pub fn hover_bg(dark: bool) -> Color32 {
    if dark {
        Color32::from_rgb(0x3A, 0x3A, 0x3A)
    } else {
        Color32::from_rgb(0xE4, 0xEF, 0xE8)
    }
}

/// Palette for the chart series ("matplotlib" style), readable on both themes.
pub const SERIES_COLORS: [Color32; 8] = [
    Color32::from_rgb(0x1F, 0x77, 0xB4),
    Color32::from_rgb(0xD6, 0x27, 0x28),
    Color32::from_rgb(0x2C, 0xA0, 0x2C),
    Color32::from_rgb(0xFF, 0x7F, 0x0E),
    Color32::from_rgb(0x94, 0x67, 0xBD),
    Color32::from_rgb(0x8C, 0x56, 0x4B),
    Color32::from_rgb(0x17, 0xBE, 0xCF),
    Color32::from_rgb(0xE3, 0x77, 0xC2),
];

pub fn series_color(i: usize) -> Color32 {
    SERIES_COLORS[i % SERIES_COLORS.len()]
}

// ---------------------------------------------------------------------------
// Applying the style
// ---------------------------------------------------------------------------

pub fn apply(ctx: &egui::Context, dark: bool) {
    let mut visuals = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    let a = accent(dark);

    visuals.panel_fill = panel_bg(dark);
    visuals.window_fill = card_bg(dark);
    visuals.extreme_bg_color = if dark {
        Color32::from_rgb(0x15, 0x15, 0x15)
    } else {
        Color32::WHITE
    };
    visuals.faint_bg_color = if dark {
        Color32::from_rgb(0x24, 0x24, 0x24)
    } else {
        Color32::from_rgb(0xF0, 0xF0, 0xF0)
    };
    visuals.selection.bg_fill = a;
    visuals.selection.stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.hyperlink_color = a;
    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.menu_corner_radius = CornerRadius::same(6);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, border(dark));
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, visuals.text_color());
    for w in [
        &mut visuals.widgets.inactive,
        &mut visuals.widgets.hovered,
        &mut visuals.widgets.active,
        &mut visuals.widgets.open,
    ] {
        w.corner_radius = CornerRadius::same(4);
    }
    visuals.widgets.inactive.weak_bg_fill = if dark {
        Color32::from_rgb(0x33, 0x33, 0x33)
    } else {
        Color32::from_rgb(0xEC, 0xEC, 0xEC)
    };
    visuals.widgets.hovered.weak_bg_fill = hover_bg(dark);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, a.gamma_multiply(0.75));
    visuals.widgets.active.bg_fill = a.gamma_multiply(0.9);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.active.bg_stroke = Stroke::new(1.0_f32, a);

    let mut style = (*ctx.style()).clone();
    style.visuals = visuals;
    style.spacing.item_spacing = Vec2::new(6.0, 4.0);
    style.spacing.button_padding = Vec2::new(6.0, 3.0);
    style.spacing.interact_size.y = 20.0;
    style.spacing.menu_margin = egui::Margin::same(4);
    // "Solid" scroll bars (not floating): they reserve space inside
    // themselves, so they do not cover the last row of tables and lists.
    // With the default egui style the bars float above the content.
    let mut scroll = egui::style::ScrollStyle::solid();
    scroll.bar_width = 8.0;
    scroll.bar_inner_margin = 4.0;
    scroll.bar_outer_margin = 2.0;
    style.spacing.scroll = scroll;
    // More compact text: it reduces the overall footprint of the interface.
    style.text_styles = [
        (egui::TextStyle::Heading, FontId::proportional(16.0)),
        (egui::TextStyle::Body, FontId::proportional(12.5)),
        (egui::TextStyle::Monospace, FontId::monospace(11.5)),
        (egui::TextStyle::Button, FontId::proportional(12.5)),
        (egui::TextStyle::Small, FontId::proportional(10.5)),
    ]
    .into();
    ctx.set_style(style);
}

// ---------------------------------------------------------------------------
// Vector icons
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Plus,
    Save,
    Trash,
    Search,
    Folder,
    Grid,
    Chart,
    Pdf,
    Refresh,
    Pencil,
    Sun,
    Moon,
    Info,
    Close,
    Points,
    Legend,
    Cube,
    Eye,
    EyeOff,
}

/// Converts normalized coordinates (0..1) of the icon rectangle into screen points.
fn n(rect: Rect, x: f32, y: f32) -> egui::Pos2 {
    egui::pos2(rect.left() + rect.width() * x, rect.top() + rect.height() * y)
}

/// Draws a vector icon inside `rect` (no dependency on fonts).
pub fn draw_icon(painter: &egui::Painter, icon: Icon, rect: Rect, color: Color32, w: f32) {
    let s = Stroke::new(w, color);
    let line = |a: (f32, f32), b: (f32, f32)| {
        painter.line_segment([n(rect, a.0, a.1), n(rect, b.0, b.1)], s);
    };
    // Almond-shaped outline of the eye, shared by the two variants.
    let mandorla = || {
        painter.add(egui::Shape::closed_line(
            vec![
                n(rect, 0.05, 0.50),
                n(rect, 0.32, 0.27),
                n(rect, 0.68, 0.27),
                n(rect, 0.95, 0.50),
                n(rect, 0.68, 0.73),
                n(rect, 0.32, 0.73),
            ],
            s,
        ));
    };

    match icon {
        Icon::Plus => {
            line((0.5, 0.18), (0.5, 0.82));
            line((0.18, 0.5), (0.82, 0.5));
        }
        Icon::Save => {
            // "Floppy" body
            painter.rect_stroke(
                Rect::from_min_max(n(rect, 0.16, 0.16), n(rect, 0.84, 0.84)),
                CornerRadius::same(1),
                s,
                egui::StrokeKind::Middle,
            );
            // Label at the top
            painter.rect_filled(
                Rect::from_min_max(n(rect, 0.32, 0.16), n(rect, 0.68, 0.40)),
                CornerRadius::ZERO,
                color,
            );
            // Shutter at the bottom
            painter.rect_stroke(
                Rect::from_min_max(n(rect, 0.30, 0.55), n(rect, 0.70, 0.84)),
                CornerRadius::ZERO,
                s,
                egui::StrokeKind::Middle,
            );
        }
        Icon::Trash => {
            line((0.16, 0.28), (0.84, 0.28));
            line((0.40, 0.18), (0.60, 0.18));
            painter.add(egui::Shape::closed_line(
                vec![
                    n(rect, 0.24, 0.28),
                    n(rect, 0.76, 0.28),
                    n(rect, 0.69, 0.84),
                    n(rect, 0.31, 0.84),
                ],
                s,
            ));
            line((0.42, 0.42), (0.42, 0.72));
            line((0.58, 0.42), (0.58, 0.72));
        }
        Icon::Search => {
            painter.circle_stroke(n(rect, 0.42, 0.42), rect.width() * 0.26, s);
            line((0.62, 0.62), (0.84, 0.84));
        }
        Icon::Folder => {
            painter.add(egui::Shape::closed_line(
                vec![
                    n(rect, 0.12, 0.78),
                    n(rect, 0.12, 0.22),
                    n(rect, 0.40, 0.22),
                    n(rect, 0.47, 0.34),
                    n(rect, 0.88, 0.34),
                    n(rect, 0.88, 0.78),
                ],
                s,
            ));
        }
        Icon::Grid => {
            painter.rect_stroke(
                Rect::from_min_max(n(rect, 0.15, 0.15), n(rect, 0.85, 0.85)),
                CornerRadius::same(1),
                s,
                egui::StrokeKind::Middle,
            );
            line((0.15, 0.38), (0.85, 0.38));
            line((0.15, 0.62), (0.85, 0.62));
            line((0.48, 0.15), (0.48, 0.85));
            line((0.70, 0.15), (0.70, 0.85));
        }
        Icon::Chart => {
            line((0.16, 0.14), (0.16, 0.84));
            line((0.16, 0.84), (0.86, 0.84));
            line((0.28, 0.62), (0.44, 0.36));
            line((0.44, 0.36), (0.58, 0.54));
            line((0.58, 0.54), (0.80, 0.26));
        }
        Icon::Pdf => {
            painter.rect_stroke(
                Rect::from_min_max(n(rect, 0.24, 0.12), n(rect, 0.76, 0.88)),
                CornerRadius::same(1),
                s,
                egui::StrokeKind::Middle,
            );
            line((0.36, 0.32), (0.64, 0.32));
            // "export" arrow pointing down
            line((0.50, 0.42), (0.50, 0.66));
            line((0.38, 0.56), (0.50, 0.68));
            line((0.62, 0.56), (0.50, 0.68));
        }
        Icon::Refresh => {
            painter.circle_stroke(n(rect, 0.5, 0.5), rect.width() * 0.30, s);
            painter.add(egui::Shape::convex_polygon(
                vec![
                    n(rect, 0.72, 0.06),
                    n(rect, 0.96, 0.30),
                    n(rect, 0.62, 0.36),
                ],
                color,
                Stroke::NONE,
            ));
        }
        Icon::Pencil => {
            painter.add(egui::Shape::closed_line(
                vec![
                    n(rect, 0.20, 0.80),
                    n(rect, 0.28, 0.56),
                    n(rect, 0.68, 0.16),
                    n(rect, 0.84, 0.32),
                    n(rect, 0.44, 0.72),
                ],
                s,
            ));
            line((0.28, 0.56), (0.44, 0.72));
            line((0.20, 0.80), (0.28, 0.72));
        }
        Icon::Sun => {
            painter.circle_filled(n(rect, 0.5, 0.5), rect.width() * 0.20, color);
            for i in 0..8 {
                let a = std::f32::consts::TAU * (i as f32) / 8.0;
                let (dx, dy) = (a.cos(), a.sin());
                line(
                    (0.5 + dx * 0.30, 0.5 + dy * 0.30),
                    (0.5 + dx * 0.44, 0.5 + dy * 0.44),
                );
            }
        }
        Icon::Moon => {
            // Crescent: arc of the outer circle + arc of the inner circle (shifted).
            let (cx, cy, r) = (0.5f32, 0.5f32, 0.36f32);
            let (bx, by, br) = (0.62f32, 0.5f32, 0.30f32);
            let d = ((bx - cx).powi(2) + (by - cy).powi(2)).sqrt();
            let a = (d * d + r * r - br * br) / (2.0 * d);
            let h = (r * r - a * a).max(0.0).sqrt();
            let ix = cx + a * (bx - cx) / d;
            let iy = cy + a * (by - cy) / d;
            let p1 = (ix, iy - h);
            let p2 = (ix, iy + h);
            let ang = |px: f32, py: f32, ox: f32, oy: f32| (py - oy).atan2(px - ox);
            let a1 = ang(p1.0, p1.1, cx, cy);
            let a2 = ang(p2.0, p2.1, cx, cy);
            let b1 = ang(p2.0, p2.1, bx, by);
            let b2 = ang(p1.0, p1.1, bx, by);

            let mut pts: Vec<egui::Pos2> = Vec::new();
            // outer arc passing along the left side (angles decreasing from a1)
            let mut t = a1;
            while t > a2 - std::f32::consts::TAU {
                pts.push(n(rect, cx + r * t.cos(), cy + r * t.sin()));
                t -= 0.15;
            }
            pts.push(n(rect, p2.0, p2.1));
            // inner arc passing along the left side of the inner circle
            let mut t = b1;
            let mut guard = 0;
            while (t - b2).abs() > 0.05 && guard < 200 {
                pts.push(n(rect, bx + br * t.cos(), by + br * t.sin()));
                t += 0.15;
                guard += 1;
            }
            painter.add(egui::Shape::convex_polygon(pts, color, Stroke::NONE));
        }
        Icon::Info => {
            painter.circle_stroke(n(rect, 0.5, 0.5), rect.width() * 0.36, s);
            line((0.5, 0.44), (0.5, 0.72));
            painter.circle_filled(n(rect, 0.5, 0.30), w * 0.7, color);
        }
        Icon::Close => {
            line((0.24, 0.24), (0.76, 0.76));
            line((0.76, 0.24), (0.24, 0.76));
        }
        Icon::Points => {
            line((0.12, 0.78), (0.88, 0.30));
            for (px, py) in [(0.22, 0.68), (0.50, 0.54), (0.78, 0.40)] {
                painter.circle_filled(n(rect, px, py), rect.width() * 0.10, color);
            }
        }
        Icon::Legend => {
            for i in 0..3 {
                let y = 0.24 + i as f32 * 0.26;
                painter.rect_filled(
                    Rect::from_min_max(n(rect, 0.12, y - 0.06), n(rect, 0.30, y + 0.06)),
                    CornerRadius::same(1),
                    color,
                );
                line((0.40, y), (0.88, y));
            }
        }
        Icon::Cube => {
            // Simplified isometric parallelepiped (front face + top face).
            let (tl, tr, br, bl) = (
                (0.20, 0.36),
                (0.66, 0.36),
                (0.66, 0.82),
                (0.20, 0.82),
            );
            let (ttl, ttr, tbr) = ((0.36, 0.20), (0.82, 0.20), (0.82, 0.66));
            painter.add(egui::Shape::closed_line(
                vec![
                    n(rect, tl.0, tl.1),
                    n(rect, tr.0, tr.1),
                    n(rect, br.0, br.1),
                    n(rect, bl.0, bl.1),
                ],
                s,
            ));
            // Top face
            line(tl, ttl);
            line(tr, ttr);
            line(ttl, ttr);
            // Right side face
            line(ttr, tbr);
            line(br, tbr);
        }
        Icon::Eye => {
            // Open eye: almond shape with the pupil.
            mandorla();
            painter.circle_stroke(n(rect, 0.5, 0.5), rect.width() * 0.15, s);
        }
        Icon::EyeOff => {
            // Crossed-out eye: the same shape crossed diagonally.
            mandorla();
            line((0.14, 0.86), (0.86, 0.14));
        }
    }
}

// ---------------------------------------------------------------------------
// Reusable widgets
// ---------------------------------------------------------------------------

/// "Large" ribbon button: icon above, label below.
pub fn big_button(ui: &mut egui::Ui, icon: Icon, label: &str) -> egui::Response {
    big_button_state(ui, icon, label, false)
}

/// Like [`big_button`], but with the "active" (toggle) state highlighted.
pub fn big_button_state(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    active: bool,
) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let a = accent(dark);
    let size = Vec2::new(48.0, 38.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let painter = ui.painter();

    // The active state ("Points" and "Legend" of the command bar) is indicated
    // **only** by the frame, like every other selection in the program; the
    // fill is reserved for transient pointer feedback.
    let bg = if resp.is_pointer_button_down_on() {
        a.gamma_multiply(0.30)
    } else if resp.hovered() {
        hover_bg(dark)
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        painter.rect_filled(rect, CornerRadius::same(5), bg);
    }
    if active {
        selection_frame(painter, rect, CornerRadius::same(5), dark);
    } else if resp.hovered() {
        painter.rect_stroke(
            rect,
            CornerRadius::same(5),
            Stroke::new(1.0_f32, a.gamma_multiply(0.55)),
            egui::StrokeKind::Inside,
        );
    }

    let icon_rect = Rect::from_center_size(
        egui::pos2(rect.center().x, rect.top() + 13.0),
        Vec2::splat(17.0),
    );
    let fg = if resp.hovered() || active {
        a
    } else {
        ui.visuals().text_color()
    };
    draw_icon(painter, icon, icon_rect, fg, 1.5);
    painter.text(
        egui::pos2(rect.center().x, rect.bottom() - 9.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(10.0),
        ui.visuals().text_color(),
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Compact "pill" button: optional icon + text, with active state.
pub fn pill_button(
    ui: &mut egui::Ui,
    icon: Option<Icon>,
    label: &str,
    active: bool,
) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let a = accent(dark);
    let text_w = ui
        .fonts(|f| {
            f.layout_no_wrap(
                label.to_owned(),
                FontId::proportional(11.0),
                Color32::WHITE,
            )
            .size()
            .x
        });
    let width = text_w + 16.0 + if icon.is_some() { 15.0 } else { 0.0 };
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(width, 20.0), egui::Sense::click());
    let painter = ui.painter();

    // The selection (Settings tabs, active chart buttons) is **only** a frame:
    // no fill, otherwise the text would have to be forced to white on the
    // light theme and would lose its own color.
    let bg = if resp.is_pointer_button_down_on() {
        a.gamma_multiply(0.35)
    } else if resp.hovered() {
        hover_bg(dark)
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        painter.rect_filled(rect, CornerRadius::same(4), bg);
    }
    painter.rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(
            1.4_f32,
            if active {
                a
            } else {
                border(dark)
            },
        ),
        egui::StrokeKind::Inside,
    );

    let fg = if active {
        a
    } else if resp.hovered() {
        a
    } else {
        ui.visuals().text_color()
    };
    let mut text_left = rect.left() + 8.0;
    if let Some(ic) = icon {
        let icon_rect = Rect::from_center_size(
            egui::pos2(rect.left() + 12.0, rect.center().y),
            Vec2::splat(12.0),
        );
        draw_icon(painter, ic, icon_rect, fg, 1.3);
        text_left += 14.0;
    }
    painter.text(
        egui::pos2(text_left, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.0),
        fg,
    );
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// White "card" with a border, used to group the contents of a page.
pub fn card<R>(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let dark = ui.visuals().dark_mode;
    egui::Frame::new()
        .fill(card_bg(dark))
        .stroke(Stroke::new(1.0_f32, border(dark)))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(egui::Margin::same(9))
        .show(ui, add)
        .inner
}

/// Section title with an accent bar on the left.
pub fn section_title(ui: &mut egui::Ui, title: &str) {
    let dark = ui.visuals().dark_mode;
    let a = accent(dark);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(3.0, 15.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, CornerRadius::same(2), a);
        ui.add_space(2.0);
        ui.label(
            RichText::new(title)
                .size(13.0)
                .strong()
                // Full black in the light theme: egui's default text color in
                // the light theme is a gray (80), not black.
                .color(text_strong(dark)),
        );
    });
}

/// Secondary label (gray, small).
pub fn hint(ui: &mut egui::Ui, text: &str) {
    let dark = ui.visuals().dark_mode;
    ui.label(RichText::new(text).size(10.5).color(muted(dark)));
}

/// Selection frame: it is the **only** way the program indicates a
/// selection (project list row, sensor list entry, active-tab label,
/// active button). The selection is always and only an outline, never a
/// fill: this way the underlying text keeps its own color and stays
/// readable, and all tabs are recognized in the same way.
pub fn selection_frame(
    painter: &egui::Painter,
    rect: egui::Rect,
    corner: egui::CornerRadius,
    dark: bool,
) {
    painter.rect_stroke(
        rect.expand(2.0),
        corner,
        Stroke::new(1.4_f32, accent(dark)),
        egui::StrokeKind::Inside,
    );
}

/// Message box (success / error) in "banner" style.
pub fn message_banner(ui: &mut egui::Ui, text: &str, is_error: bool) {
    let dark = ui.visuals().dark_mode;
    let (fg, bg) = if is_error {
        (
            if dark { Color32::from_rgb(0xFF, 0xB4, 0xAB) } else { Color32::from_rgb(0x8B, 0x1A, 0x0F) },
            DANGER.gamma_multiply(if dark { 0.30 } else { 0.12 }),
        )
    } else {
        (
            if dark { Color32::from_rgb(0xA5, 0xE8, 0xC0) } else { Color32::from_rgb(0x14, 0x5A, 0x32) },
            OK.gamma_multiply(if dark { 0.30 } else { 0.14 }),
        )
    };
    egui::Frame::new()
        .fill(bg)
        .stroke(Stroke::new(1.0_f32, fg.gamma_multiply(0.6)))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(8, 5))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).color(fg));
        });
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Primary,
    Danger,
    Neutral,
}

/// Action button with icon + text: solid (primary), red (danger) or neutral.
/// Small square button with a vector icon, meant for the rows of a
/// list where a scrolling action is needed (sizes and pointer
/// feedback similar to `Ui::small_button`).
pub fn small_icon_button(
    ui: &mut egui::Ui,
    icon: Icon,
    color: Color32,
    size: f32,
) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::click());
    if resp.hovered() || resp.is_pointer_button_down_on() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(3), hover_bg(dark));
    }
    draw_icon(ui.painter(), icon, rect, color, 1.4);
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

pub fn action_button(
    ui: &mut egui::Ui,
    icon: Icon,
    label: &str,
    kind: ButtonKind,
) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let a = accent(dark);
    let text_w = ui.fonts(|f| {
        f.layout_no_wrap(label.to_owned(), FontId::proportional(11.5), Color32::WHITE)
            .size()
            .x
    });
    let size = Vec2::new(text_w + 34.0, 23.0);
    let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
    let painter = ui.painter();

    let (bg, fg, border_color) = match kind {
        ButtonKind::Primary => (a, Color32::WHITE, a),
        ButtonKind::Danger => (DANGER, Color32::WHITE, DANGER),
        ButtonKind::Neutral => (
            if resp.hovered() {
                hover_bg(dark)
            } else {
                Color32::TRANSPARENT
            },
            if resp.hovered() {
                a
            } else {
                ui.visuals().text_color()
            },
            border(dark),
        ),
    };

    let bg = if resp.is_pointer_button_down_on() {
        bg.gamma_multiply(0.85)
    } else if resp.hovered() && kind != ButtonKind::Neutral {
        bg.gamma_multiply(1.15)
    } else {
        bg
    };

    if bg != Color32::TRANSPARENT {
        painter.rect_filled(rect, CornerRadius::same(4), bg);
    }
    painter.rect_stroke(
        rect,
        CornerRadius::same(4),
        Stroke::new(1.0_f32, border_color),
        egui::StrokeKind::Inside,
    );

    let icon_rect = Rect::from_center_size(
        egui::pos2(rect.left() + 12.0, rect.center().y),
        Vec2::splat(12.0),
    );
    draw_icon(painter, icon, icon_rect, fg, 1.4);
    painter.text(
        egui::pos2(rect.left() + 22.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(11.5),
        fg,
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp
}

/// Field label with fixed width, right-aligned ("professional" form style).
pub fn field_label(ui: &mut egui::Ui, text: &str, width: f32) {
    ui.add_sized(
        Vec2::new(width, 18.0),
        egui::Label::new(RichText::new(text).size(11.5)).halign(egui::Align::RIGHT),
    );
}
