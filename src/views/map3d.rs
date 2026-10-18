//! "3D Sensors" tab: a rotatable parallelepiped on which to place the project's
//! sensors, with preview and PDF export of the visualization.
//!
//! Positions are saved in the SQLite database (table `sensor_positions`),
//! one per project and per sensor: the same sensor cannot be placed
//! twice (a constraint enforced both in the interface and in the database).

use crate::data;
use crate::i18n::{self, Language};
use crate::models::{Project, SceneDims, SensorPos};
use crate::scene3d::{self, Camera};
use crate::theme::{self, ButtonKind, Icon};
use crate::views::ViewState;
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Rect, RichText, Stroke, StrokeKind, Vec2,
};

/// Data of the map's PDF preview (they stay in memory as long as it is open).
pub struct MapPreview {
    pub project: Project,
    pub dims: SceneDims,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub positions: Vec<SensorPos>,
    pub stats: Vec<(String, f32, f32)>,
}

/// Colors used by the scene drawing (screen or paper).
struct MapStyle {
    bg: Color32,
    face: Color32,
    edge: Color32,
    label_bg: Color32,
    label_fg: Color32,
}

/// Style for the interactive view, consistent with the light/dark theme.
///
/// The container is rendered semi-transparent, so that the sensors (even on
/// far faces) stay visible: each face is a gray with a reduced alpha channel,
/// drawn from the background toward the camera.
fn screen_style(dark: bool) -> MapStyle {
    if dark {
        MapStyle {
            bg: Color32::from_rgb(0x1A, 0x1A, 0x1C),
            face: Color32::from_rgba_unmultiplied(0xC2, 0xCA, 0xD2, 50),
            edge: Color32::from_rgb(0xD0, 0xD8, 0xE0),
            label_bg: Color32::from_black_alpha(185),
            label_fg: Color32::from_rgb(0xF2, 0xF2, 0xF2),
        }
    } else {
        MapStyle {
            bg: Color32::from_rgb(0xFA, 0xFA, 0xFC),
            face: Color32::from_rgba_unmultiplied(0x54, 0x5C, 0x66, 90),
            edge: Color32::from_rgb(0x3E, 0x46, 0x50),
            label_bg: Color32::from_black_alpha(185),
            label_fg: Color32::WHITE,
        }
    }
}

/// "Paper" style for the PDF preview (opaque: it represents the printed sheet).
fn paper_style() -> MapStyle {
    MapStyle {
        bg: Color32::from_rgb(0xFA, 0xFA, 0xFA),
        face: Color32::from_rgb(0xD9, 0xDD, 0xE1),
        edge: Color32::from_rgb(0x60, 0x60, 0x60),
        label_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        label_fg: Color32::from_rgb(0x20, 0x20, 0x20),
    }
}

/// Minimum and maximum of each sensor derived from the data file series.
fn sensor_stats(series: &[data::Series]) -> Vec<(String, f32, f32)> {
    series
        .iter()
        .map(|s| {
            let mut lo = f32::INFINITY;
            let mut hi = f32::NEG_INFINITY;
            for p in &s.points {
                lo = lo.min(p[1] as f32);
                hi = hi.max(p[1] as f32);
            }
            if s.points.is_empty() {
                (s.name.clone(), f32::NAN, f32::NAN)
            } else {
                (s.name.clone(), lo, hi)
            }
        })
        .collect()
}

/// Multiplies a color by a light factor (0..1), preserving the alpha
/// (needed for the semi-transparent container).
fn shade(c: Color32, k: f32) -> Color32 {
    let f = |v: u8| (v as f32 * k).clamp(0.0, 255.0) as u8;
    Color32::from_rgba_unmultiplied(f(c.r()), f(c.g()), f(c.b()), c.a())
}

impl ViewState {
    /// Draws the "Sensors 3D" tab.
    pub fn show_map_ui(&mut self, ui: &mut egui::Ui, db: &crate::models::Database) {
        let dark = ui.visuals().dark_mode;
        let lang = self.lang;

        let Some(proj) = self.active_project.clone() else {
            theme::card(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.add_space(16.0);
                    ui.label(
                        RichText::new(i18n::t(lang, "Nessun progetto attivo selezionato."))
                            .size(12.0)
                            .color(theme::DANGER),
                    );
                    ui.label(
                        RichText::new(i18n::t(
                            lang,
                            "Apri un progetto dalla scheda Ricerca per posizionare i sensori.",
                        ))
                        .size(11.0)
                        .color(theme::muted(dark)),
                    );
                    ui.add_space(16.0);
                });
            });
            return;
        };

        // Loads layout and positions from the database when the active project changes.
        if self.map_loaded_for != Some(proj.id) {
            self.map_dims = db.load_scene_dims(proj.id).unwrap_or_default().sanitized();
            self.map_positions = db.load_sensor_positions(proj.id);
            self.map_selected = None;
            // Any drag in progress belonged to the previous project.
            self.map_dragging = None;
            // Visibility is a session preference: when the project
            // changes, all sensors become visible again.
            self.map_hidden.clear();
            let cam = Camera::default();
            self.map_yaw = cam.yaw;
            self.map_pitch = cam.pitch;
            self.map_zoom = cam.zoom;
            self.map_loaded_for = Some(proj.id);
        }

        // Available sensors and their thermal extremes, from the project's data file.
        if self.csv_rows.is_empty() && self.csv_load_error.is_none() {
            self.load_data_file();
        }
        let series = data::parse_series(&self.csv_headers, &self.csv_rows);
        let stats = sensor_stats(&series);

        // --- Local copies manipulated by the UI, applied at the end of the frame ---
        let mut dims = self.map_dims;
        let mut yaw = self.map_yaw;
        let mut pitch = self.map_pitch;
        let mut zoom = self.map_zoom;
        let mut positions = self.map_positions.clone();
        let mut selected = self.map_selected.clone();
        // Sensors hidden from the diagram, toggled by the eye in the list.
        let mut hidden = self.map_hidden.clone();

        let mut dims_changed = false;
        let mut place: Option<(usize, f32, f32)> = None;
        let mut remove: Option<String> = None;
        let mut clear_all = false;
        let mut reset_view = false;
        let mut open_preview = false;

        // --- Dragging an already placed point ---
        // Starts by pressing on an existing point and ends when the button is
        // released: as long as `dragging` is set the canvas does not rotate the scene.
        let mut dragging: Option<String> = self.map_dragging.clone();
        // The new coordinates coming out of the closures, to be written at frame end.
        let mut drag_moved: Option<(String, scene3d::Vec3)> = None;
        // Name of the released sensor: at that moment the position must be saved.
        let mut drag_release: Option<String> = None;

        // --- Command bar ---
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&proj.name)
                        .size(13.5)
                        .strong()
                        .color(theme::text_strong(dark)),
                );
                let n = positions.len();
                let noun = i18n::plural(lang, n, "sensore posizionato", "sensori posizionati");
                ui.label(
                    RichText::new(format!("{n} {noun}"))
                        .size(11.0)
                        .color(theme::muted(dark)),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::action_button(
                        ui,
                        Icon::Pdf,
                        i18n::t(lang, "Anteprima PDF"),
                        ButtonKind::Primary,
                    )
                    .on_hover_text(i18n::t(
                        lang,
                        "Mostra l'anteprima della mappa dei sensori in un PDF A4",
                    ))
                    .clicked()
                    {
                        open_preview = true;
                    }
                    if theme::action_button(
                        ui,
                        Icon::Refresh,
                        i18n::t(lang, "Reimposta vista"),
                        ButtonKind::Neutral,
                    )
                    .on_hover_text(i18n::t(lang, "Riporta la rotazione e lo zoom iniziali"))
                    .clicked()
                    {
                        reset_view = true;
                    }
                    if theme::action_button(
                        ui,
                        Icon::Trash,
                        i18n::t(lang, "Rimuovi tutti"),
                        ButtonKind::Danger,
                    )
                    .on_hover_text(i18n::t(lang, "Rimuove tutti i sensori dalla mappa"))
                    .clicked()
                    {
                        clear_all = true;
                    }
                });
            });
        });

        ui.add_space(8.0);

        if let Some(err) = &self.csv_load_error {
            theme::message_banner(ui, err, true);
            ui.add_space(6.0);
        }

        let avail = ui.available_size();
        let left_w = 300.0_f32.min(avail.x * 0.45).max(200.0);
        let body_h = avail.y.max(320.0);

        ui.horizontal_top(|ui| {
            // ------------------- Left panel: controls -------------------
            ui.allocate_ui_with_layout(
                Vec2::new(left_w, body_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    theme::card(ui, |ui| {
                        ui.set_width(ui.available_width());
                        theme::section_title(ui, i18n::t(lang, "Dimensioni container (m)"));
                        ui.add_space(4.0);
                        egui::Grid::new("map_dims_grid")
                            .num_columns(2)
                            .spacing([10.0, 5.0])
                            .show(ui, |ui| {
                                ui.label(i18n::t(lang, "Larghezza"));
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut dims.w)
                                            .speed(0.05)
                                            .range(0.5..=60.0),
                                    )
                                    .changed()
                                {
                                    dims_changed = true;
                                }
                                ui.end_row();
                                ui.label(i18n::t(lang, "Altezza"));
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut dims.h)
                                            .speed(0.05)
                                            .range(0.5..=60.0),
                                    )
                                    .changed()
                                {
                                    dims_changed = true;
                                }
                                ui.end_row();
                                ui.label(i18n::t(lang, "Profondità"));
                                if ui
                                    .add(
                                        egui::DragValue::new(&mut dims.d)
                                            .speed(0.05)
                                            .range(0.5..=60.0),
                                    )
                                    .changed()
                                {
                                    dims_changed = true;
                                }
                                ui.end_row();
                            });
                    });

                    ui.add_space(6.0);

                    theme::card(ui, |ui| {
                        ui.set_width(ui.available_width());
                        theme::section_title(ui, i18n::t(lang, "Sensori del progetto"));
                        ui.add_space(4.0);

                        if stats.is_empty() {
                            theme::hint(
                                ui,
                                i18n::t(
                                    lang,
                                    "Nessun sensore trovato: apri i dati del progetto per elencarli.",
                                ),
                            );
                        } else {
                            // The hint goes above the list because the list
                            // must be able to occupy, at every frame, all the space that
                            // remains in the card: this way its height is deduced
                            // from the last position reached by the layout, without
                            // magic constants that, as the window shrinks,
                            // end up cutting it off.
                            theme::hint(
                                ui,
                                i18n::t(
                                    lang,
                                    "Seleziona un sensore e clicca sul parallelepipedo per posizionarlo. Trascina un punto per spostarlo, il fondo per ruotare, la rotella per lo zoom, il tasto destro su un punto per rimuoverlo.",
                                ),
                            );
                            ui.add_space(4.0);

                            // Everything that remains: if the sensors do not fit, the
                            // scroll bar appears and the list scrolls
                            // vertically; if they fit, it fills exactly the card.
                            let list_h = ui.available_height().max(96.0);
                            egui::ScrollArea::vertical()
                                .max_height(list_h)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    for (name, lo, hi) in &stats {
                                        let is_placed =
                                            positions.iter().any(|p| &p.name == name);
                                        ui.horizontal(|ui| {
                                            let is_sel =
                                                selected.as_deref() == Some(name.as_str());
                                            // The name keeps the text color (or green if already
                                            // placed): the selection is shown only by a
                                            // frame, so that the name always stays legible.
                                            let mut txt = RichText::new(name).size(11.5);
                                            txt = if is_placed {
                                                txt.color(theme::OK)
                                            } else {
                                                txt.color(theme::text_strong(dark))
                                            };
                                            if is_sel {
                                                txt = txt.strong();
                                            }
                                            let resp = ui.selectable_label(false, txt);
                                            if is_sel {
                                                theme::selection_frame(
                                                    ui.painter(),
                                                    resp.rect,
                                                    egui::CornerRadius::same(3),
                                                    dark,
                                                );
                                            }
                                            if resp.clicked() {
                                                selected = Some(name.clone());
                                            }
                                            let temps = if lo.is_finite() {
                                                format!("{lo:.1} / {hi:.1} °C")
                                            } else {
                                                "-".to_string()
                                            };
                                            ui.label(
                                                RichText::new(temps)
                                                    .size(10.0)
                                                    .color(theme::muted(dark)),
                                            );
                                            if is_placed {
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        // With a right-to-left layout the first
                                                        // widget takes up the far right: removal
                                                        // stays where it was and the eye
                                                        // sits right next to it.
                                                        if ui
                                                            .small_button("×")
                                                            .on_hover_text(i18n::t(
                                                                lang,
                                                                "Rimuovi questo sensore",
                                                            ))
                                                            .clicked()
                                                        {
                                                            remove = Some(name.clone());
                                                        }
                                                        let nascosto = hidden.contains(name);
                                                        let icona = if nascosto {
                                                            Icon::EyeOff
                                                        } else {
                                                            Icon::Eye
                                                        };
                                                        let colore = if nascosto {
                                                            theme::muted(dark)
                                                        } else {
                                                            ui.visuals().text_color()
                                                        };
                                                        let occhio =
                                                            theme::small_icon_button(ui, icona, colore, 18.0);
                                                        let testo = if nascosto {
                                                            "Mostra questo sensore nel diagramma"
                                                        } else {
                                                            "Nascondi questo sensore dal diagramma"
                                                        };
                                                        if occhio
                                                            .on_hover_text(i18n::t(lang, testo))
                                                            .clicked()
                                                        {
                                                            if nascosto {
                                                                hidden.remove(name);
                                                            } else {
                                                                hidden.insert(name.clone());
                                                            }
                                                        }
                                                    },
                                                );
                                            }
                                        });
                                    }
                                });
                        }
                    });
                },
            );

            ui.separator();

            // ------------------- 3D canvas -------------------
            let w = ui.available_width().max(200.0);
            ui.allocate_ui_with_layout(
                Vec2::new(w, body_h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(w, body_h), egui::Sense::click_and_drag());

                    let cam = Camera { yaw, pitch, zoom };
                    let proj = scene3d::Projector::new(
                        rect.center().x,
                        rect.center().y,
                        rect.width().min(rect.height()),
                        &cam,
                        dims,
                    );

                    // --- Dragging an already placed point ---
                    // It engages only if, when the drag begins, the
                    // cursor is still inside the point's area (14 px): this way it
                    // grabs the point you have under your fingers, while the scene
                    // starts rotating only when pressing on empty space.
                    if response.drag_started() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            if let Some(name) =
                                nearest_marker(pos, rect, dims, yaw, pitch, zoom, &positions, &hidden)
                            {
                                dragging = Some(name);
                            }
                        }
                    }

                    if let Some(name) = dragging.clone() {
                        // The point slides in the plane parallel to the view that
                        // passes through it: it arrives exactly where you point and,
                        // if the camera looks from a three-quarter angle, it moves with
                        // x, y and z components together. It is not snapped back onto
                        // any wall, so it can detach from the container.
                        if let Some(p) = positions.iter_mut().find(|p| p.name == name) {
                            if let Some(ptr) = response.interact_pointer_pos() {
                                let punto = scene3d::sensor_point(p, dims);
                                let normalizzato =
                                    scene3d::normalize_point(proj.drag_at_depth(punto, ptr.x, ptr.y), dims);
                                p.pos = Some(normalizzato);
                                drag_moved = Some((name.clone(), normalizzato));
                            }
                        }
                        if response.drag_stopped() {
                            drag_release = Some(name);
                            dragging = None;
                        }
                    } else if response.dragged() {
                        // Rotation: drag with the left button.
                        let d = response.drag_delta();
                        yaw += d.x * 0.01;
                        pitch = (pitch + d.y * 0.01).clamp(-1.45, 1.45);
                    }

                    let style = screen_style(dark);
                    draw_scene(
                        ui.painter(),
                        rect,
                        dims,
                        yaw,
                        pitch,
                        zoom,
                        &positions,
                        &hidden,
                        &stats,
                        &style,
                    );

                    // Zoom with the wheel.
                    if response.hovered() {
                        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
                        if scroll.abs() > 0.0 {
                            zoom = (zoom * (1.0 + scroll * 0.0015)).clamp(0.35, 3.5);
                        }
                        ui.ctx().set_cursor_icon(if dragging.is_some() {
                            egui::CursorIcon::Grabbing
                        } else {
                            egui::CursorIcon::Grab
                        });
                    }
                    // Placement: click on a face. While a point is in
                    // hand the click places nothing.
                    if response.clicked() && dragging.is_none() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let (o, d) = proj.unproject(pos.x, pos.y);
                            if let Some((face, u, v)) = scene3d::pick_face(o, d, dims) {
                                place = Some((face, u, v));
                            }
                        }
                    }
                    // Removal: right-click on an existing point.
                    if response.secondary_clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            if let Some(name) =
                                nearest_marker(pos, rect, dims, yaw, pitch, zoom, &positions, &hidden)
                            {
                                remove = Some(name);
                            }
                        }
                    }
                },
            );
        });

        // ------------------- Apply the changes -------------------
        self.map_dims = dims.sanitized();
        self.map_yaw = yaw;
        self.map_pitch = pitch;
        self.map_zoom = zoom;
        self.map_selected = selected;
        // The drag continues from one frame to the next until the
        // button is released.
        self.map_dragging = dragging;
        // State of the list's eye (session only).
        self.map_hidden = hidden.clone();

        // Position updated during the drag: it is first written into the
        // list, so that the next frame restarts from here.
        if let Some((name, pos)) = drag_moved {
            if let Some(p) = self.map_positions.iter_mut().find(|p| p.name == name) {
                p.pos = Some(pos);
            }
        }
        // Release: the final position also goes to the database, otherwise
        // it would be lost on closing.
        if let Some(name) = drag_release {
            if let Some(p) = self.map_positions.iter().find(|p| p.name == name) {
                let pos = p.clone();
                if let Err(e) = db.update_sensor_position(proj.id, &pos) {
                    self.set_status(format!("{} {e}", i18n::t(lang, "Errore SQLite:")), true);
                }
            }
        }

        if dims_changed {
            if let Err(e) = db.save_scene_dims(proj.id, self.map_dims) {
                self.set_status(format!("{} {e}", i18n::t(lang, "Errore SQLite:")), true);
            }
        }

        if let Some((face, u, v)) = place {
            match self.map_selected.clone() {
                None => self.set_status(
                    i18n::t(lang, "Seleziona un sensore dall'elenco prima di cliccare."),
                    true,
                ),
                Some(name) => {
                    if self.map_positions.iter().any(|p| p.name == name) {
                        self.set_status(
                            format!("{} {name}", i18n::t(lang, "Sensore già posizionato:")),
                            true,
                        );
                    } else {
                        let pos = SensorPos {
                            name: name.clone(),
                            face,
                            u,
                            v,
                            // Still glued to the wall: no actual
                            // position until it is dragged.
                            pos: None,
                        };
                        match db.insert_sensor_position(proj.id, &pos) {
                            Ok(()) => {
                                self.map_positions.push(pos);
                                // After placement the sensor is deselected.
                                self.map_selected = None;
                                self.set_status(
                                    format!("{} {name}", i18n::t(lang, "Sensore posizionato:")),
                                    false,
                                );
                            }
                            Err(_) => self.set_status(
                                format!("{} {name}", i18n::t(lang, "Sensore già posizionato:")),
                                true,
                            ),
                        }
                    }
                }
            }
        }

        if let Some(name) = remove {
            match db.delete_sensor_position(proj.id, &name) {
                Ok(()) => {
                    self.map_positions.retain(|p| p.name != name);
                    self.set_status(
                        format!("{} {name}", i18n::t(lang, "Sensore rimosso:")),
                        false,
                    );
                }
                Err(e) => self.set_status(format!("{} {e}", i18n::t(lang, "Errore delete:")), true),
            }
        }

        if clear_all {
            match db.delete_all_sensor_positions(proj.id) {
                Ok(()) => {
                    self.map_positions.clear();
                    self.set_status(i18n::t(lang, "Mappa svuotata: sensori rimossi."), false);
                }
                Err(e) => self.set_status(format!("{} {e}", i18n::t(lang, "Errore delete:")), true),
            }
        }

        if reset_view {
            self.reset_map_view();
        }

        if open_preview {
            self.open_map_preview();
        }
    }

    /// Brings the map camera back to the initial framing.
    pub fn reset_map_view(&mut self) {
        let cam = Camera::default();
        self.map_yaw = cam.yaw;
        self.map_pitch = cam.pitch;
        self.map_zoom = cam.zoom;
        self.set_status(i18n::t(self.lang, "Vista reimpostata."), false);
    }

    /// Prepares the PDF preview of the sensors map.
    pub fn open_map_preview(&mut self) {
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
        let stats = sensor_stats(&series);

        if self.map_positions.is_empty() {
            self.set_status(
                i18n::t(lang, "Posiziona almeno un sensore prima di esportare la mappa."),
                true,
            );
            return;
        }

        // The preview and the PDF portray the scene just as it appears on screen:
        // sensors turned off by the eye do not appear there either.
        let visibili: Vec<SensorPos> = self
            .map_positions
            .iter()
            .filter(|p| !self.map_hidden.contains(&p.name))
            .cloned()
            .collect();
        if visibili.is_empty() {
            self.set_status(
                i18n::t(
                    lang,
                    "Tutti i sensori sono nascosti: mostrane almeno uno per esportare.",
                ),
                true,
            );
            return;
        }

        self.map_preview = Some(MapPreview {
            project,
            dims: self.map_dims.sanitized(),
            yaw: self.map_yaw,
            pitch: self.map_pitch,
            zoom: self.map_zoom,
            positions: visibili,
            stats,
        });
        self.set_status(i18n::t(lang, "Anteprima mappa pronta: controlla e salva."), false);
    }

    /// Draws the map preview window (if open).
    pub fn show_map_preview_window(&mut self, ctx: &egui::Context) {
        let Some(preview) = self.map_preview.take() else {
            return;
        };
        let lang = self.lang;

        let mut open = true;
        let mut do_save = false;
        let mut do_close = false;

        egui::Window::new(i18n::t(lang, "Anteprima PDF — Mappa sensori 3D"))
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
                    let (pw_mm, ph_mm) = crate::export::PAGE_SIZE_MM;
                    let page_h = page_w * (ph_mm / pw_mm);
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::new(page_w, page_h), egui::Sense::hover());
                    draw_map_page(ui.painter(), rect, &preview, lang);
                });
            });

        if do_save {
            self.write_map_pdf_with_dialog(&preview);
            self.map_preview = None;
            return;
        }
        if do_close || !open {
            self.set_status(i18n::t(lang, "Anteprima mappa chiusa."), false);
            return;
        }
        self.map_preview = Some(preview);
    }
}

/// Finds the sensor closest (in pixels) to a clicked point, within a threshold.
/// Hidden sensors are not considered: they are not grabbed, they are not
/// dragged and they are not removed with the right button while they are off.
fn nearest_marker(
    pos: egui::Pos2,
    rect: Rect,
    dims: SceneDims,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    positions: &[SensorPos],
    hidden: &std::collections::BTreeSet<String>,
) -> Option<String> {
    let cam = Camera { yaw, pitch, zoom };
    let proj = scene3d::Projector::new(
        rect.center().x,
        rect.center().y,
        rect.width().min(rect.height()),
        &cam,
        dims,
    );
    let mut best: Option<(f32, &str)> = None;
    for p in positions {
        if hidden.contains(&p.name) {
            continue;
        }
        let (x, y) = proj.project(scene3d::sensor_point(p, dims));
        let d = ((x - pos.x).powi(2) + (y - pos.y).powi(2)).sqrt();
        if d <= 14.0 && best.map_or(true, |(bd, _)| d < bd) {
            best = Some((d, p.name.as_str()));
        }
    }
    best.map(|(_, n)| n.to_string())
}

/// Draws the 3D scene (parallelepiped + sensors) inside `rect`.
fn draw_scene(
    painter: &egui::Painter,
    rect: Rect,
    dims: SceneDims,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    positions: &[SensorPos],
    hidden: &std::collections::BTreeSet<String>,
    stats: &[(String, f32, f32)],
    style: &MapStyle,
) {
    painter.rect_filled(rect, CornerRadius::same(4), style.bg);

    let viewport = rect.width().min(rect.height());
    let cam = Camera { yaw, pitch, zoom };
    let proj = scene3d::Projector::new(
        rect.center().x,
        rect.center().y,
        viewport,
        &cam,
        dims,
    );

    // Faces, from the background toward the camera.
    for q in scene3d::face_quads(&proj, dims) {
        let pts: Vec<egui::Pos2> = q.pts.iter().map(|(x, y)| egui::pos2(*x, *y)).collect();
        painter.add(egui::Shape::convex_polygon(
            pts,
            shade(style.face, q.shade),
            Stroke::new(1.0_f32, style.edge),
        ));
    }

    // Sensors with name and minimum/maximum temperatures. Hidden sensors
    // are skipped, but the series index remains the one of the full
    // list: a sensor's color does not change by hiding another one.
    for (i, p) in positions.iter().enumerate() {
        if hidden.contains(&p.name) {
            continue;
        }
        let (x, y) = proj.project(scene3d::sensor_point(p, dims));
        let center = egui::pos2(x, y);
        let color = theme::series_color(i);
        painter.circle_filled(center, 5.0, color);
        painter.circle_stroke(center, 5.0, Stroke::new(1.2_f32, Color32::WHITE));

        let (lo, hi) = stats
            .iter()
            .find(|(n, _, _)| *n == p.name)
            .map(|(_, a, b)| (*a, *b))
            .unwrap_or((f32::NAN, f32::NAN));
        draw_label(painter, center + Vec2::new(9.0, -10.0), &p.name, lo, hi, style);
    }
}

/// Label with the sensor name and minimum/maximum temperatures.
fn draw_label(
    painter: &egui::Painter,
    anchor: egui::Pos2,
    name: &str,
    lo: f32,
    hi: f32,
    style: &MapStyle,
) {
    let name_g = painter.layout_no_wrap(name.to_string(), FontId::proportional(11.0), style.label_fg);
    let temps = if lo.is_finite() {
        format!("{lo:.1} / {hi:.1} °C")
    } else {
        "-".to_string()
    };
    let temp_g = painter.layout_no_wrap(temps, FontId::proportional(10.0), style.label_fg);

    let w = name_g.size().x.max(temp_g.size().x);
    let h = name_g.size().y + temp_g.size().y;
    let bg = Rect::from_min_size(anchor, Vec2::new(w, h)).expand(3.0);
    painter.rect_filled(bg, CornerRadius::same(3), style.label_bg);
    painter.rect_stroke(
        bg,
        CornerRadius::same(3),
        Stroke::new(0.6_f32, style.label_fg.gamma_multiply(0.35)),
        StrokeKind::Inside,
    );
    let name_h = name_g.size().y;
    painter.galley(anchor, name_g, style.label_fg);
    painter.galley(anchor + Vec2::new(0.0, name_h), temp_g, style.label_fg);
}

/// Draws the A4 page of the preview, replicating the PDF layout.
fn draw_map_page(painter: &egui::Painter, page: Rect, preview: &MapPreview, lang: Language) {
    use crate::export::{BOTTOM, LEFT, RIGHT, TOP};

    let (pw_mm, ph_mm) = crate::export::PAGE_SIZE_MM;
    let sx = page.width() / pw_mm;

    let mm = |x: f32, y: f32| {
        egui::pos2(
            page.left() + x * sx,
            page.top() + (ph_mm - y) * (page.height() / ph_mm),
        )
    };
    let fs = |pt: f32| (pt * 0.352_778 * sx).max(4.5);

    let ink = Color32::from_rgb(0x20, 0x20, 0x20);
    let muted_c = Color32::from_rgb(0x70, 0x70, 0x70);
    let accent = Color32::from_rgb(0x21, 0x73, 0x46);

    // Sheet
    painter.rect_filled(page, CornerRadius::ZERO, Color32::WHITE);
    painter.rect_stroke(
        page,
        CornerRadius::ZERO,
        Stroke::new(1.0_f32, Color32::from_rgb(0xC8, 0xC8, 0xC8)),
        StrokeKind::Inside,
    );

    // Header
    painter.text(
        mm(LEFT, 199.0),
        egui::Align2::LEFT_BOTTOM,
        i18n::t(lang, "Mappa sensori 3D"),
        FontId::proportional(fs(17.0)),
        ink,
    );
    painter.text(
        mm(LEFT, 192.5),
        egui::Align2::LEFT_BOTTOM,
        &preview.project.name,
        FontId::proportional(fs(12.0)),
        accent,
    );

    let info = format!(
        "{}: {}    {}: {:.1} × {:.1} × {:.1} m    {}: {}",
        i18n::t(lang, "Container"),
        if preview.project.container_type.trim().is_empty() {
            "-"
        } else {
            preview.project.container_type.as_str()
        },
        i18n::t(lang, "Dimensioni"),
        preview.dims.w,
        preview.dims.h,
        preview.dims.d,
        i18n::t(lang, "Sensori"),
        preview.positions.len(),
    );
    painter.text(
        mm(LEFT, 187.0),
        egui::Align2::LEFT_BOTTOM,
        info,
        FontId::proportional(fs(8.5)),
        muted_c,
    );
    painter.text(
        mm(RIGHT, 187.0),
        egui::Align2::RIGHT_BOTTOM,
        format!(
            "{} {}",
            i18n::t(lang, "Generato il"),
            chrono::Local::now().format("%d/%m/%Y %H:%M")
        ),
        FontId::proportional(fs(8.5)),
        muted_c,
    );

    // Scene area. The preview positions have already been filtered
    // when the preview was opened, so no sensor is excluded here.
    let area = Rect::from_two_pos(mm(LEFT, BOTTOM), mm(RIGHT, TOP));
    draw_scene(
        painter,
        area,
        preview.dims,
        preview.yaw,
        preview.pitch,
        preview.zoom,
        &preview.positions,
        &std::collections::BTreeSet::new(),
        &preview.stats,
        &paper_style(),
    );

    // Footer
    painter.text(
        mm(LEFT, 11.0),
        egui::Align2::LEFT_BOTTOM,
        i18n::t(lang, "Thermite · Mappatura Termica di Trasporto"),
        FontId::proportional(fs(7.0)),
        muted_c,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A sensor turned off by the eye also disappears from the interactions:
    /// while it is hidden it is not grabbed with the pointer (so neither selection,
    /// nor dragging, nor removal with the right button), and it becomes available
    /// again as soon as it is shown again.
    #[test]
    fn nearest_marker_ignora_i_sensori_nascosti() {
        let dims = SceneDims {
            w: 6.0,
            h: 2.5,
            d: 2.4,
        };
        let rect = Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(400.0, 300.0));
        let cam = Camera::default();
        let proiettore = scene3d::Projector::new(
            rect.center().x,
            rect.center().y,
            rect.width().min(rect.height()),
            &cam,
            dims,
        );

        let posizioni = vec![SensorPos {
            name: "S1".to_string(),
            face: 4,
            u: 0.5,
            v: 0.5,
            pos: None,
        }];
        let (x, y) = proiettore.project(scene3d::sensor_point(&posizioni[0], dims));

        // No hidden sensor: the point is grabbed exactly.
        let nessuno = std::collections::BTreeSet::new();
        assert_eq!(
            nearest_marker(
                egui::pos2(x, y),
                rect,
                dims,
                cam.yaw,
                cam.pitch,
                cam.zoom,
                &posizioni,
                &nessuno,
            ),
            Some("S1".to_string()),
        );

        // Hidden: the point stops responding to the pointer.
        let nascosti: std::collections::BTreeSet<String> =
            ["S1".to_string()].into_iter().collect();
        assert_eq!(
            nearest_marker(
                egui::pos2(x, y),
                rect,
                dims,
                cam.yaw,
                cam.pitch,
                cam.zoom,
                &posizioni,
                &nascosti,
            ),
            None,
            "un sensore nascosto non deve essere afferrabile"
        );
    }
}
