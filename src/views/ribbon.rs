//! Excel / ArtCraft style ribbon: menu bar, tabs, command groups with icons,
//! status bar and the "About Thermite" window.

use crate::i18n::{self, Language};
use crate::models::{Command, SelectedTab};
use crate::theme::{self, Icon};
use crate::views::ViewState;
use eframe::egui::{self, Align2, Color32, CornerRadius, FontId, RichText, Vec2};

/// Draws the menu bar + the tab ribbon + the command groups.
///
/// The background is not drawn here: the top panel provides it, following the
/// current theme (light/dark). Here we only add the Office-style separator
/// lines.
pub fn show_top_bar(ui: &mut egui::Ui, state: &ViewState) -> Option<Command> {
    let dark = ui.visuals().dark_mode;
    let lang = state.lang;
    let mut cmd: Option<Command> = show_menu_bar(ui, lang);

    // --- Tab row ---
    ui.horizontal(|ui| {
        for tab in SelectedTab::ALL {
            if ribbon_tab(ui, tab, state.current_tab, lang).clicked() {
                cmd = Some(Command::SwitchTab(tab));
            }
        }
        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| {
                if theme::pill_button(ui, Some(Icon::Info), i18n::t(lang, "Informazioni"), false)
                    .clicked()
                {
                    cmd = Some(Command::About);
                }
            },
        );
    });

    hairline(ui, dark);

    // --- Command groups of the current tab ---
    if let Some(group_cmd) = show_groups(ui, state) {
        cmd = Some(group_cmd);
    }

    // Bottom border of the ribbon, to set it apart from the work area.
    hairline(ui, dark);

    cmd
}

/// Full-width thin line, used as a separator between the ribbon areas.
fn hairline(ui: &mut egui::Ui, dark: bool) {
    ui.add_space(1.0);
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::ZERO, theme::border(dark));
    ui.add_space(1.0);
}

fn show_menu_bar(ui: &mut egui::Ui, lang: Language) -> Option<Command> {
    let mut cmd = None;
    egui::menu::bar(ui, |ui| {
        ui.menu_button(i18n::t(lang, "File"), |ui| {
            if ui.button(i18n::t(lang, "Nuovo progetto")).clicked() {
                cmd = Some(Command::NewProject);
                ui.close_menu();
            }
            if ui.button(i18n::t(lang, "Apri file dati…")).clicked() {
                cmd = Some(Command::PickDataFile);
                ui.close_menu();
            }
            if ui.button(i18n::t(lang, "Ricarica dati sensori")).clicked() {
                cmd = Some(Command::ReloadData);
                ui.close_menu();
            }
            ui.separator();
            if ui
                .button(i18n::t(lang, "Esporta grafico in PDF…"))
                .on_hover_text(i18n::t(
                    lang,
                    "Mostra l'anteprima del grafico termico in un PDF A4",
                ))
                .clicked()
            {
                cmd = Some(Command::ExportPdf);
                ui.close_menu();
            }
            if ui
                .button(i18n::t(lang, "Esporta mappa sensori in PDF…"))
                .on_hover_text(i18n::t(
                    lang,
                    "Mostra l'anteprima della mappa dei sensori in un PDF A4",
                ))
                .clicked()
            {
                cmd = Some(Command::ExportMapPdf);
                ui.close_menu();
            }
            ui.separator();
            if ui.button(i18n::t(lang, "Configurazioni…")).clicked() {
                cmd = Some(Command::OpenSettings);
                ui.close_menu();
            }
            ui.separator();
            if ui.button(i18n::t(lang, "Esci")).clicked() {
                cmd = Some(Command::Quit);
                ui.close_menu();
            }
        });

        ui.menu_button(i18n::t(lang, "Visualizza"), |ui| {
            let dark = ui.visuals().dark_mode;
            if ui
                .button(if dark {
                    i18n::t(lang, "Passa al tema chiaro")
                } else {
                    i18n::t(lang, "Passa al tema scuro")
                })
                .clicked()
            {
                cmd = Some(Command::ToggleTheme);
                ui.close_menu();
            }
            if ui.button(i18n::t(lang, "Aggiorna elenco progetti")).clicked() {
                cmd = Some(Command::ReloadProjects);
                ui.close_menu();
            }
        });

        ui.menu_button(i18n::t(lang, "Vai"), |ui| {
            for tab in SelectedTab::ALL {
                if ui.button(tab.label(lang)).clicked() {
                    cmd = Some(Command::SwitchTab(tab));
                    ui.close_menu();
                }
            }
        });

        ui.menu_button(i18n::t(lang, "Aiuto"), |ui| {
            if ui.button(i18n::t(lang, "Informazioni su Thermite")).clicked() {
                cmd = Some(Command::About);
                ui.close_menu();
            }
        });
    });
    cmd
}

/// A ribbon tab; the active one is highlighted with the accent color and a bar.
fn ribbon_tab(
    ui: &mut egui::Ui,
    tab: SelectedTab,
    current: SelectedTab,
    lang: Language,
) -> egui::Response {
    let dark = ui.visuals().dark_mode;
    let a = theme::accent(dark);
    let active = tab == current;

    let text_w = ui.fonts(|f| {
        f.layout_no_wrap(
            tab.label(lang).to_owned(),
            FontId::proportional(11.5),
            Color32::WHITE,
        )
        .size()
        .x
    });
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(text_w + 24.0, 22.0), egui::Sense::click());
    let painter = ui.painter();

    // The active tab is identified **only** by its frame: no fill and no
    // underline, like every other selection in the program. The pointer
    // highlight over the tab remains, but it is transient feedback and not the
    // selection indicator — which is why it is not drawn on the active tab,
    // which must look like a frame and nothing more.
    let bg = if !active && resp.hovered() {
        theme::hover_bg(dark)
    } else {
        Color32::TRANSPARENT
    };
    if bg != Color32::TRANSPARENT {
        painter.rect_filled(rect, CornerRadius::same(5), bg);
    }
    if active {
        theme::selection_frame(painter, rect, CornerRadius::same(5), dark);
    }
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        tab.label(lang),
        FontId::proportional(11.5),
        if active { a } else { ui.visuals().text_color() },
    );

    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    resp.on_hover_text(tab.description(lang))
}

/// Command groups of the active tab.
fn show_groups(ui: &mut egui::Ui, state: &ViewState) -> Option<Command> {
    let mut cmd = None;
    let lang = state.lang;
    let width = ui.available_width();

    ui.allocate_ui_with_layout(
        Vec2::new(width, 48.0),
        egui::Layout::left_to_right(egui::Align::TOP),
        |ui| {
            match state.current_tab {
                SelectedTab::Management => {
                    group(ui, i18n::t(lang, "Progetto"), |ui| {
                        if theme::big_button(ui, Icon::Plus, i18n::t(lang, "Nuovo"))
                            .on_hover_text(i18n::t(
                                lang,
                                "Svuota il modulo per inserire un nuovo progetto",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::NewProject);
                        }
                        if theme::big_button(ui, Icon::Save, i18n::t(lang, "Salva"))
                            .on_hover_text(i18n::t(
                                lang,
                                "Salva il progetto nel database (nuovo o modificato)",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::SaveProject);
                        }
                        if theme::big_button(ui, Icon::Trash, i18n::t(lang, "Elimina"))
                            .on_hover_text(i18n::t(
                                lang,
                                "Elimina il progetto attualmente in modifica",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::DeleteProject);
                        }
                    });
                    group(ui, i18n::t(lang, "Archivio"), |ui| {
                        if theme::big_button(ui, Icon::Folder, i18n::t(lang, "Apri file"))
                            .on_hover_text(i18n::t(
                                lang,
                                "Collega un file CSV/Excel con i dati dei sensori",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::PickDataFile);
                        }
                        if theme::big_button(ui, Icon::Refresh, i18n::t(lang, "Aggiorna"))
                            .on_hover_text(i18n::t(
                                lang,
                                "Ricarica l'elenco dei progetti dal database",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::ReloadProjects);
                        }
                    });
                    navigation_group(ui, lang, &mut cmd);
                    export_group(ui, lang, &mut cmd);
                }

                SelectedTab::SearchList => {
                    group(ui, i18n::t(lang, "Ricerca"), |ui| {
                        if theme::big_button(ui, Icon::Close, i18n::t(lang, "Pulisci"))
                            .on_hover_text(i18n::t(lang, "Azzera il testo di ricerca"))
                            .clicked()
                        {
                            cmd = Some(Command::ClearSearch);
                        }
                    });
                    group(ui, i18n::t(lang, "Elenco"), |ui| {
                        if theme::big_button(ui, Icon::Refresh, i18n::t(lang, "Aggiorna"))
                            .on_hover_text(i18n::t(lang, "Ricarica l'elenco dei progetti"))
                            .clicked()
                        {
                            cmd = Some(Command::ReloadProjects);
                        }
                        if theme::big_button(ui, Icon::Plus, i18n::t(lang, "Nuovo")).clicked() {
                            cmd = Some(Command::NewProject);
                        }
                    });
                    navigation_group(ui, lang, &mut cmd);
                    export_group(ui, lang, &mut cmd);
                }

                SelectedTab::DataGrid => {
                    group(ui, i18n::t(lang, "File dati"), |ui| {
                        if theme::big_button(ui, Icon::Folder, i18n::t(lang, "Apri file"))
                            .on_hover_text(i18n::t(
                                lang,
                                "Collega un file CSV/Excel al progetto",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::PickDataFile);
                        }
                        if theme::big_button(ui, Icon::Refresh, i18n::t(lang, "Ricarica"))
                            .on_hover_text(i18n::t(
                                lang,
                                "Rileggi il file dati del progetto attivo",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::ReloadData);
                        }
                    });
                    navigation_group(ui, lang, &mut cmd);
                    export_group(ui, lang, &mut cmd);
                }

                SelectedTab::ChartPlot => {
                    group(ui, i18n::t(lang, "Esporta"), |ui| {
                        if theme::big_button(ui, Icon::Pdf, "PDF")
                            .on_hover_text(i18n::t(
                                lang,
                                "Esporta il grafico termico in un documento PDF A4",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::ExportPdf);
                        }
                    });
                    group(ui, i18n::t(lang, "Serie"), |ui| {
                        if theme::big_button_state(ui, Icon::Points, i18n::t(lang, "Punti"), state.show_points)
                            .on_hover_text(i18n::t(
                                lang,
                                "Mostra o nascondi i punti campionati",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::TogglePoints);
                        }
                        if theme::big_button_state(ui, Icon::Legend, i18n::t(lang, "Legenda"), state.show_legend)
                            .on_hover_text(i18n::t(lang, "Mostra o nascondi la legenda"))
                            .clicked()
                        {
                            cmd = Some(Command::ToggleLegend);
                        }
                    });
                    group(ui, i18n::t(lang, "Vista"), |ui| {
                        if theme::big_button(ui, Icon::Refresh, "Zoom")
                            .on_hover_text(i18n::t(
                                lang,
                                "Reimposta zoom e spostamento del grafico",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::ResetZoom);
                        }
                        if theme::big_button(ui, Icon::Grid, i18n::t(lang, "Griglia"))
                            .on_hover_text(i18n::t(lang, "Torna alla griglia dei dati"))
                            .clicked()
                        {
                            cmd = Some(Command::SwitchTab(SelectedTab::DataGrid));
                        }
                    });
                    group(ui, i18n::t(lang, "File dati"), |ui| {
                        if theme::big_button(ui, Icon::Folder, i18n::t(lang, "Apri file")).clicked() {
                            cmd = Some(Command::PickDataFile);
                        }
                        if theme::big_button(ui, Icon::Refresh, i18n::t(lang, "Ricarica")).clicked() {
                            cmd = Some(Command::ReloadData);
                        }
                    });
                }

                SelectedTab::SensorMap => {
                    group(ui, i18n::t(lang, "Esporta"), |ui| {
                        if theme::big_button(ui, Icon::Pdf, "PDF")
                            .on_hover_text(i18n::t(
                                lang,
                                "Esporta la mappa dei sensori in un PDF A4",
                            ))
                            .clicked()
                        {
                            cmd = Some(Command::ExportMapPdf);
                        }
                    });
                    group(ui, i18n::t(lang, "Vista"), |ui| {
                        if theme::big_button(ui, Icon::Refresh, i18n::t(lang, "Reimposta"))
                            .on_hover_text(i18n::t(lang, "Riporta la rotazione e lo zoom iniziali"))
                            .clicked()
                        {
                            cmd = Some(Command::ResetMapView);
                        }
                        if theme::big_button(ui, Icon::Grid, i18n::t(lang, "Griglia"))
                            .on_hover_text(i18n::t(lang, "Torna alla griglia dei dati"))
                            .clicked()
                        {
                            cmd = Some(Command::SwitchTab(SelectedTab::DataGrid));
                        }
                        if theme::big_button(ui, Icon::Chart, i18n::t(lang, "Grafico"))
                            .on_hover_text(i18n::t(lang, "Grafico termico del progetto attivo"))
                            .clicked()
                        {
                            cmd = Some(Command::SwitchTab(SelectedTab::ChartPlot));
                        }
                    });
                    group(ui, i18n::t(lang, "File dati"), |ui| {
                        if theme::big_button(ui, Icon::Folder, i18n::t(lang, "Apri file")).clicked() {
                            cmd = Some(Command::PickDataFile);
                        }
                        if theme::big_button(ui, Icon::Refresh, i18n::t(lang, "Ricarica")).clicked() {
                            cmd = Some(Command::ReloadData);
                        }
                    });
                }
            }
        },
    );

    cmd
}

fn navigation_group(ui: &mut egui::Ui, lang: Language, cmd: &mut Option<Command>) {
    group(ui, i18n::t(lang, "Visualizza"), |ui| {
        if theme::big_button(ui, Icon::Grid, i18n::t(lang, "Griglia"))
            .on_hover_text(i18n::t(lang, "Griglia dei dati del progetto attivo"))
            .clicked()
        {
            *cmd = Some(Command::SwitchTab(SelectedTab::DataGrid));
        }
        if theme::big_button(ui, Icon::Chart, i18n::t(lang, "Grafico"))
            .on_hover_text(i18n::t(lang, "Grafico termico del progetto attivo"))
            .clicked()
        {
            *cmd = Some(Command::SwitchTab(SelectedTab::ChartPlot));
        }
        if theme::big_button(ui, Icon::Cube, i18n::t(lang, "Sensori 3D"))
            .on_hover_text(i18n::t(lang, "Posizione dei sensori nel container 3D"))
            .clicked()
        {
            *cmd = Some(Command::SwitchTab(SelectedTab::SensorMap));
        }
    });
}

fn export_group(ui: &mut egui::Ui, lang: Language, cmd: &mut Option<Command>) {
    group(ui, i18n::t(lang, "Esporta"), |ui| {
        if theme::big_button(ui, Icon::Pdf, "PDF")
            .on_hover_text(i18n::t(lang, "Esporta il grafico termico in PDF"))
            .clicked()
        {
            *cmd = Some(Command::ExportPdf);
        }
    });
}

/// A command group with a centered caption and a vertical separator.
fn group(ui: &mut egui::Ui, title: &str, add: impl FnOnce(&mut egui::Ui)) {
    let dark = ui.visuals().dark_mode;
    let rect = ui
        .vertical(|ui| {
            ui.horizontal(|ui| {
                ui.add_space(2.0);
                add(ui);
                ui.add_space(2.0);
            });
            // space reserved for the caption
            ui.add_space(10.0);
        })
        .response
        .rect;

    ui.painter().text(
        egui::pos2(rect.center().x, rect.bottom() - 5.0),
        Align2::CENTER_CENTER,
        title,
        FontId::proportional(9.0),
        theme::muted(dark),
    );

    ui.add(egui::Separator::default().vertical().spacing(6.0));
}

/// Status bar at the bottom, with the active project, counts and messages.
pub fn show_status_bar(ui: &mut egui::Ui, state: &ViewState, project_count: usize) {
    let dark = ui.visuals().dark_mode;
    let muted = theme::muted(dark);
    let lang = state.lang;

    ui.horizontal(|ui| {
        match &state.status {
            Some((msg, is_error)) => {
                let color = if *is_error { theme::DANGER } else { theme::OK };
                ui.label(RichText::new(msg).size(10.5).color(color));
            }
            None => {
                ui.label(RichText::new(i18n::t(lang, "Pronto")).size(10.5).color(muted));
            }
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                RichText::new(format!(
                    "{} · {}",
                    state.current_tab.label(lang),
                    state.current_tab.description(lang)
                ))
                .size(10.5)
                .color(muted),
            );
            ui.separator();
            ui.label(
                RichText::new(format!(
                    "{project_count} {}",
                    i18n::plural(lang, project_count, "progetto", "progetti")
                ))
                .size(10.5)
                .color(muted),
            );
            if !state.csv_rows.is_empty() {
                let n = state.csv_rows.len();
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "{n} {}",
                        i18n::plural(lang, n, "riga dati", "righe dati")
                    ))
                    .size(10.5)
                    .color(muted),
                );
            }
            if let Some(p) = &state.active_project {
                ui.separator();
                ui.label(
                    RichText::new(format!(
                        "{} {} (#{})",
                        i18n::t(lang, "Progetto attivo:"),
                        p.name,
                        p.id
                    ))
                    .size(10.5)
                    .color(muted),
                );
            }
        });
    });
}

/// "Informazioni su Thermite" modal window.
pub fn show_about_window(ctx: &egui::Context, state: &mut ViewState) {
    if !state.show_about {
        return;
    }
    let dark = ctx.style().visuals.dark_mode;
    let lang = state.lang;
    let mut open = true;
    egui::Window::new(i18n::t(lang, "Informazioni su Thermite"))
        .open(&mut open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.set_min_width(320.0);
            ui.vertical_centered(|ui| {
                ui.add_space(6.0);
                ui.label(RichText::new("Thermite").size(20.0).strong());
                ui.label(
                    RichText::new(i18n::t(lang, "Mappatura Termica di Trasporto"))
                        .size(11.5)
                        .color(theme::muted(dark)),
                );
                ui.add_space(6.0);
            });
            ui.separator();
            ui.label(
                RichText::new(i18n::t(
                    lang,
                    "Applicazione desktop in Rust (egui / eframe) per gestire la mappatura termica di trasporto, l'analisi dei dati dei sensori (CSV / Excel) e la produzione di grafici e mappa 3D delle posizioni dei sensori.",
                ))
                .size(11.5),
            );
            ui.add_space(10.0);
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(i18n::t(
                        lang,
                        "Software pubblicato sotto licenza\nGNU GENERAL PUBLIC LICENSE\nVersion 3, 29 June 2007",
                    ))
                    .size(10.5)
                    .color(theme::muted(dark)),
                );
            });
        });
    if !open {
        state.show_about = false;
    }
}
