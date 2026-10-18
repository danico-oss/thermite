//! Scheda "Sensori 3D": parallelepipedo ruotabile su cui posizionare i sensori
//! del progetto, con anteprima ed esportazione in PDF della visualizzazione.
//!
//! Le posizioni sono salvate nel database SQLite (tabella `sensor_positions`),
//! una per progetto e per sensore: lo stesso sensore non può essere posizionato
//! due volte (vincolo applicato sia nell'interfaccia sia nel database).

use crate::data;
use crate::i18n::{self, Language};
use crate::models::{Project, SceneDims, SensorPos};
use crate::scene3d::{self, Camera};
use crate::theme::{self, ButtonKind, Icon};
use crate::views::ViewState;
use eframe::egui::{
    self, Color32, CornerRadius, FontId, Rect, RichText, Stroke, StrokeKind, Vec2,
};

/// Dati dell'anteprima PDF della mappa (restano in memoria finché è aperta).
pub struct MapPreview {
    pub project: Project,
    pub dims: SceneDims,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub positions: Vec<SensorPos>,
    pub stats: Vec<(String, f32, f32)>,
}

/// Colori usati dal disegno della scena (schermo o carta).
struct MapStyle {
    bg: Color32,
    face: Color32,
    edge: Color32,
    label_bg: Color32,
    label_fg: Color32,
}

/// Stile per la vista interattiva, coerente con il tema chiaro/scuro.
///
/// Il container è reso semitrasparente, così i sensori (anche su facce lontane)
/// restano visibili: ogni faccia è un grigio con canale alfa ridotto, disegnata
/// dal fondo verso la telecamera.
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

/// Stile "carta" per l'anteprima del PDF (opaco: rappresenta il foglio stampato).
fn paper_style() -> MapStyle {
    MapStyle {
        bg: Color32::from_rgb(0xFA, 0xFA, 0xFA),
        face: Color32::from_rgb(0xD9, 0xDD, 0xE1),
        edge: Color32::from_rgb(0x60, 0x60, 0x60),
        label_bg: Color32::from_rgb(0xFF, 0xFF, 0xFF),
        label_fg: Color32::from_rgb(0x20, 0x20, 0x20),
    }
}

/// Minimo e massimo di ogni sensore ricavati dalle serie del file dati.
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

/// Moltiplica un colore per un fattore di luce (0..1), conservando l'alfa
/// (necessario per il container semitrasparente).
fn shade(c: Color32, k: f32) -> Color32 {
    let f = |v: u8| (v as f32 * k).clamp(0.0, 255.0) as u8;
    Color32::from_rgba_unmultiplied(f(c.r()), f(c.g()), f(c.b()), c.a())
}

impl ViewState {
    /// Disegna la scheda "Sensori 3D".
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

        // Carica layout e posizioni dal database quando cambia il progetto attivo.
        if self.map_loaded_for != Some(proj.id) {
            self.map_dims = db.load_scene_dims(proj.id).unwrap_or_default().sanitized();
            self.map_positions = db.load_sensor_positions(proj.id);
            self.map_selected = None;
            // Un eventuale trascinamento in corso apparteneva al progetto prima.
            self.map_dragging = None;
            let cam = Camera::default();
            self.map_yaw = cam.yaw;
            self.map_pitch = cam.pitch;
            self.map_zoom = cam.zoom;
            self.map_loaded_for = Some(proj.id);
        }

        // Sensori disponibili e loro estremi termici, dal file dati del progetto.
        if self.csv_rows.is_empty() && self.csv_load_error.is_none() {
            self.load_data_file();
        }
        let series = data::parse_series(&self.csv_headers, &self.csv_rows);
        let stats = sensor_stats(&series);

        // --- Copie locali manipolate dalla UI, applicate a fine frame ---
        let mut dims = self.map_dims;
        let mut yaw = self.map_yaw;
        let mut pitch = self.map_pitch;
        let mut zoom = self.map_zoom;
        let mut positions = self.map_positions.clone();
        let mut selected = self.map_selected.clone();

        let mut dims_changed = false;
        let mut place: Option<(usize, f32, f32)> = None;
        let mut remove: Option<String> = None;
        let mut clear_all = false;
        let mut reset_view = false;
        let mut open_preview = false;

        // --- Trascinamento di un punto già posizionato ---
        // Comincia premendo su un punto esistente e finisce col rilascio del
        // bottone: finché `dragging` è imposto il canvas non ruota la scena.
        let mut dragging: Option<String> = self.map_dragging.clone();
        // Le nuove coordinate uscite dalle closure, da scrivere a fine frame.
        let mut drag_moved: Option<(String, f32, f32)> = None;
        // Nome del sensore rilasciato: in quel momento la posizione va salvata.
        let mut drag_release: Option<String> = None;

        // --- Barra dei comandi ---
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
            // ------------------- Pannello sinistro: controlli -------------------
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
                            // Il suggerimento sta sopra la lista perché la lista
                            // deve poter occupare a ogni frame tutto lo spazio che
                            // resta nella card: così la sua altezza si deduce
                            // dall'ultima posizione raggiunta dal layout, senza
                            // costanti magiche che, al ridursi della finestra,
                            // finiscono per tagliarla fuori.
                            theme::hint(
                                ui,
                                i18n::t(
                                    lang,
                                    "Seleziona un sensore e clicca sul parallelepipedo per posizionarlo. Trascina un punto per spostarlo, il fondo per ruotare, la rotella per lo zoom, il tasto destro su un punto per rimuoverlo.",
                                ),
                            );
                            ui.add_space(4.0);

                            // Tutto ciò che resta: se i sensori non ci stanno, la
                            // barra di scorrimento compare e la lista scorre in
                            // verticale; se ci stanno, riempie esattamente la card.
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
                                            // Il nome mantiene il colore del testo (o verde se già
                                            // posizionato): la selezione è indicata solo da una
                                            // cornice, così il nome resta sempre leggibile.
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

            // ------------------- Canvas 3D -------------------
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

                    // --- Trascinamento di un punto già posizionato ---
                    // Si aggancia solo se, quando il trascinamento comincia, il
                    // cursore è ancora entro l'area del punto (14 px): così si
                    // afferra il punto che si ha sotto le dita, mentre la scena
                    // comincia a ruotare soltanto premendo a vuoto.
                    if response.drag_started() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            if let Some(name) =
                                nearest_marker(pos, rect, dims, yaw, pitch, zoom, &positions)
                            {
                                dragging = Some(name);
                            }
                        }
                    }

                    if let Some(name) = dragging.clone() {
                        // Il raggio del cursore colpisce la faccia che il punto
                        // occupa già: le nuove coordinate arrivano esattamente dove
                        // si punta, il punto resta su quella faccia e non ne
                        // cambia mai, quindi non "scivola" verso o lontano dalla
                        // telecamera se non per la pendenza della parete.
                        if let Some(p) = positions.iter_mut().find(|p| p.name == name) {
                            if let Some(ptr) = response.interact_pointer_pos() {
                                if let Some((u, v)) = proj.drag_on_face(p.face, ptr.x, ptr.y, dims)
                                {
                                    p.u = u;
                                    p.v = v;
                                    drag_moved = Some((name.clone(), u, v));
                                }
                            }
                        }
                        if response.drag_stopped() {
                            drag_release = Some(name);
                            dragging = None;
                        }
                    } else if response.dragged() {
                        // Rotazione: trascinamento con il tasto sinistro.
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
                        &stats,
                        &style,
                    );

                    // Zoom con la rotella.
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
                    // Posizionamento: clic su una faccia. Mentre un punto è in
                    // mano il clic non posiziona nulla.
                    if response.clicked() && dragging.is_none() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            let (o, d) = proj.unproject(pos.x, pos.y);
                            if let Some((face, u, v)) = scene3d::pick_face(o, d, dims) {
                                place = Some((face, u, v));
                            }
                        }
                    }
                    // Rimozione: clic con il tasto destro su un punto esistente.
                    if response.secondary_clicked() {
                        if let Some(pos) = response.interact_pointer_pos() {
                            if let Some(name) =
                                nearest_marker(pos, rect, dims, yaw, pitch, zoom, &positions)
                            {
                                remove = Some(name);
                            }
                        }
                    }
                },
            );
        });

        // ------------------- Applica le modifiche -------------------
        self.map_dims = dims.sanitized();
        self.map_yaw = yaw;
        self.map_pitch = pitch;
        self.map_zoom = zoom;
        self.map_selected = selected;
        // Il trascinamento continua da un frame all'altro finché non si
        // rilascia il bottone.
        self.map_dragging = dragging;

        // Posizione aggiornata durante il trascinamento: prima la si scrive nella
        // lista, così il frame successivo riparte da qui.
        if let Some((name, u, v)) = drag_moved {
            if let Some(p) = self.map_positions.iter_mut().find(|p| p.name == name) {
                p.u = u;
                p.v = v;
            }
        }
        // Rilascio: la posizione finale passa anche al database, altrimenti
        // sarebbe persa alla chiusura.
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
                        };
                        match db.insert_sensor_position(proj.id, &pos) {
                            Ok(()) => {
                                self.map_positions.push(pos);
                                // Dopo il posizionamento il sensore viene deselezionato.
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

    /// Riporta la telecamera della mappa all'inquadratura iniziale.
    pub fn reset_map_view(&mut self) {
        let cam = Camera::default();
        self.map_yaw = cam.yaw;
        self.map_pitch = cam.pitch;
        self.map_zoom = cam.zoom;
        self.set_status(i18n::t(self.lang, "Vista reimpostata."), false);
    }

    /// Prepara l'anteprima PDF della mappa dei sensori.
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

        self.map_preview = Some(MapPreview {
            project,
            dims: self.map_dims.sanitized(),
            yaw: self.map_yaw,
            pitch: self.map_pitch,
            zoom: self.map_zoom,
            positions: self.map_positions.clone(),
            stats,
        });
        self.set_status(i18n::t(lang, "Anteprima mappa pronta: controlla e salva."), false);
    }

    /// Disegna la finestra di anteprima della mappa (se aperta).
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

/// Trova il sensore più vicino (in pixel) a un punto cliccato, entro una soglia.
fn nearest_marker(
    pos: egui::Pos2,
    rect: Rect,
    dims: SceneDims,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    positions: &[SensorPos],
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
        let (x, y) = proj.project(scene3d::sensor_point(p, dims));
        let d = ((x - pos.x).powi(2) + (y - pos.y).powi(2)).sqrt();
        if d <= 14.0 && best.map_or(true, |(bd, _)| d < bd) {
            best = Some((d, p.name.as_str()));
        }
    }
    best.map(|(_, n)| n.to_string())
}

/// Disegna la scena 3D (parallelepipedo + sensori) dentro `rect`.
fn draw_scene(
    painter: &egui::Painter,
    rect: Rect,
    dims: SceneDims,
    yaw: f32,
    pitch: f32,
    zoom: f32,
    positions: &[SensorPos],
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

    // Facce, dal fondo verso la telecamera.
    for q in scene3d::face_quads(&proj, dims) {
        let pts: Vec<egui::Pos2> = q.pts.iter().map(|(x, y)| egui::pos2(*x, *y)).collect();
        painter.add(egui::Shape::convex_polygon(
            pts,
            shade(style.face, q.shade),
            Stroke::new(1.0_f32, style.edge),
        ));
    }

    // Sensori con nome e temperature minima/massima.
    for (i, p) in positions.iter().enumerate() {
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

/// Etichetta con nome del sensore e temperature minima/massima.
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

/// Disegna la pagina A4 dell'anteprima, replicando il layout del PDF.
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

    // Foglio
    painter.rect_filled(page, CornerRadius::ZERO, Color32::WHITE);
    painter.rect_stroke(
        page,
        CornerRadius::ZERO,
        Stroke::new(1.0_f32, Color32::from_rgb(0xC8, 0xC8, 0xC8)),
        StrokeKind::Inside,
    );

    // Intestazione
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

    // Area della scena
    let area = Rect::from_two_pos(mm(LEFT, BOTTOM), mm(RIGHT, TOP));
    draw_scene(
        painter,
        area,
        preview.dims,
        preview.yaw,
        preview.pitch,
        preview.zoom,
        &preview.positions,
        &preview.stats,
        &paper_style(),
    );

    // Piè di pagina
    painter.text(
        mm(LEFT, 11.0),
        egui::Align2::LEFT_BOTTOM,
        i18n::t(lang, "Thermite · Mappatura Termica di Trasporto"),
        FontId::proportional(fs(7.0)),
        muted_c,
    );
}
