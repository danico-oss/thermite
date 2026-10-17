//! Scheda "Ricerca": elenco dei progetti con filtro e tabella in stile Excel.

use crate::i18n;
use crate::models::{Project, SelectedTab};
use crate::theme::{self, ButtonKind, Icon};
use crate::views::ViewState;
use eframe::egui::{self, RichText, Vec2};
use egui_extras::{Column, TableBuilder};

impl ViewState {
    pub fn show_search_ui(&mut self, ui: &mut egui::Ui, projects: &[Project]) {
        let dark = ui.visuals().dark_mode;
        let lang = self.lang;
        let query = self.search_query.to_lowercase();
        let filtered: Vec<Project> = projects
            .iter()
            .filter(|p| {
                query.is_empty()
                    || p.name.to_lowercase().contains(&query)
                    || p.container_type.to_lowercase().contains(&query)
            })
            .cloned()
            .collect();

        // --- Barra di ricerca ---
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
                theme::draw_icon(ui.painter(), Icon::Search, rect, theme::muted(dark), 1.5);
                ui.add_space(2.0);
                ui.add_sized(
                    Vec2::new(320.0, 24.0),
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text(i18n::t(lang, "Filtra per nome o tipo di container…")),
                );
                if theme::action_button(
                    ui,
                    Icon::Close,
                    i18n::t(lang, "Pulisci"),
                    ButtonKind::Neutral,
                )
                .clicked()
                {
                    self.search_query.clear();
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let noun = i18n::plural(lang, projects.len(), "progetto", "progetti");
                    let txt = if query.is_empty() {
                        format!("{} {noun}", projects.len())
                    } else {
                        format!(
                            "{} {} {} {noun}",
                            filtered.len(),
                            i18n::t(lang, "di"),
                            projects.len()
                        )
                    };
                    ui.label(RichText::new(txt).size(11.0).color(theme::muted(dark)));
                });
            });
        });

        ui.add_space(12.0);
        theme::section_title(ui, i18n::t(lang, "Elenco progetti"));
        ui.add_space(6.0);

        if filtered.is_empty() {
            theme::card(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.add_space(18.0);
                    let (rect, _) =
                        ui.allocate_exact_size(Vec2::splat(34.0), egui::Sense::hover());
                    theme::draw_icon(ui.painter(), Icon::Folder, rect, theme::muted(dark), 2.0);
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(if projects.is_empty() {
                            i18n::t(lang, "Nessun progetto presente: creane uno dalla scheda Gestione.")
                        } else {
                            i18n::t(lang, "Nessun progetto corrisponde alla ricerca.")
                        })
                        .size(12.0)
                        .color(theme::muted(dark)),
                    );
                    ui.add_space(18.0);
                });
            });
            return;
        }

        // --- Tabella dei risultati ---
        let mut open_data: Option<Project> = None;
        let mut edit_project: Option<Project> = None;
        let mut select_project: Option<Project> = None;

        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            // Scorrimento in entrambe le direzioni: lo gestisce l'area esterna,
            // mentre la tabella interna non attiva il proprio scorrimento verticale
            // (altrimenti limita la larghezza e impedisce lo scorrimento orizzontale).
            egui::ScrollArea::both()
                .max_height(ui.available_height() - 8.0)
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .vscroll(false)
                        .sense(egui::Sense::click())
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::exact(58.0))
                        .column(Column::initial(190.0).at_least(120.0))
                        .column(Column::initial(150.0).at_least(100.0))
                        .column(Column::exact(72.0))
                        .column(Column::exact(72.0))
                        .column(Column::initial(200.0).at_least(120.0))
                        .column(Column::exact(200.0))
                        .header(26.0, |mut header| {
                            for name in [
                                "ID",
                                i18n::t(lang, "Nome"),
                                i18n::t(lang, "Container"),
                                i18n::t(lang, "Min °C"),
                                i18n::t(lang, "Max °C"),
                                i18n::t(lang, "File dati"),
                                i18n::t(lang, "Azioni"),
                            ] {
                                header.col(|ui| {
                                    ui.label(
                                        RichText::new(name)
                                            .strong()
                                            .size(11.5)
                                            .color(theme::text_strong(dark)),
                                    );
                                });
                            }
                        })
                        .body(|mut body| {
                            for p in &filtered {
                                body.row(28.0, |mut row| {
                                    row.set_selected(self.selected_edit_id == Some(p.id));

                                    row.col(|ui| {
                                        ui.label(
                                            RichText::new(format!("#{}", p.id))
                                                .size(11.0)
                                                .color(theme::muted(dark)),
                                        );
                                    });
                                    row.col(|ui| {
                                        ui.label(
                                            RichText::new(&p.name)
                                                .size(11.5)
                                                .strong()
                                                .color(theme::text_strong(dark)),
                                        );
                                    });
                                    row.col(|ui| {
                                        ui.label(RichText::new(&p.container_type).size(11.0));
                                    });
                                    row.col(|ui| {
                                        ui.label(
                                            RichText::new(format!("{:.1}", p.min_temp))
                                                .size(11.0)
                                                .color(theme::series_color(0)),
                                        );
                                    });
                                    row.col(|ui| {
                                        ui.label(
                                            RichText::new(format!("{:.1}", p.max_temp))
                                                .size(11.0)
                                                .color(theme::series_color(1)),
                                        );
                                    });
                                    row.col(|ui| {
                                        let file = file_name(&p.data_file_path);
                                        ui.label(RichText::new(file).size(10.5))
                                            .on_hover_text(&p.data_file_path);
                                    });
                                    row.col(|ui| {
                                        ui.horizontal(|ui| {
                                            if theme::pill_button(
                                                ui,
                                                Some(Icon::Chart),
                                                i18n::t(lang, "Apri dati"),
                                                false,
                                            )
                                            .on_hover_text(i18n::t(
                                                lang,
                                                "Visualizza griglia e grafico termico",
                                            ))
                                            .clicked()
                                            {
                                                open_data = Some(p.clone());
                                            }
                                            if theme::pill_button(
                                                ui,
                                                Some(Icon::Pencil),
                                                i18n::t(lang, "Modifica"),
                                                false,
                                            )
                                            .on_hover_text(i18n::t(
                                                lang,
                                                "Modifica o elimina il progetto",
                                            ))
                                            .clicked()
                                            {
                                                edit_project = Some(p.clone());
                                            }
                                        });
                                    });

                                    // Un clic su una qualunque parte della riga
                                    // (fuori dai pulsanti) seleziona il progetto.
                                    if row.response().clicked() {
                                        select_project = Some(p.clone());
                                    }
                                });
                            }
                        });
                });
        });

        if let Some(p) = open_data {
            self.activate_project_and_load(p);
            self.current_tab = SelectedTab::DataGrid;
            // L'apertura può fallire (file mancante o CRC non corrispondente):
            // in quel caso si segnala l'errore, non un successo che non c'è stato.
            match &self.csv_load_error {
                Some(e) => self.set_status(format!("{} {e}", i18n::t(lang, "Errore:")), true),
                None => {
                    self.set_status(i18n::t(lang, "Dati del progetto caricati."), false);
                }
            }
        }
        if let Some(p) = edit_project {
            self.select_project_for_operations(&p);
            self.current_tab = SelectedTab::Management;
            self.set_status(
                format!(
                    "{} #{} {}",
                    i18n::t(lang, "Progetto"),
                    p.id,
                    i18n::t(lang, "pronto per la modifica.")
                ),
                false,
            );
        }
        if let Some(p) = select_project {
            // Selezione del progetto: la riga resta evidenziata e i relativi dati
            // sono pronti per le operazioni (Gestione, apertura dati, ecc.).
            self.select_project_for_operations(&p);
            self.set_status(
                format!(
                    "{} {} (#{})",
                    i18n::t(lang, "Progetto selezionato:"),
                    p.name,
                    p.id
                ),
                false,
            );
        }
    }
}

/// Mostra solo il nome del file, non tutto il percorso.
fn file_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}
