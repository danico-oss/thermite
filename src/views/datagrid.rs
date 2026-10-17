//! Scheda "Dati": griglia tabellare dei valori letti dai sensori.

use crate::i18n;
use crate::models::SelectedTab;
use crate::theme::{self, ButtonKind, Icon};
use crate::views::ViewState;
use eframe::egui::{self, RichText};
use egui_extras::{Column, TableBuilder};

impl ViewState {
    pub fn show_datagrid_ui(&mut self, ui: &mut egui::Ui) {
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
                            "Apri un progetto dalla scheda Ricerca per vedere i dati.",
                        ))
                        .size(11.0)
                        .color(theme::muted(dark)),
                    );
                    ui.add_space(16.0);
                });
            });
            return;
        };

        // --- Intestazione con progetto attivo ---
        let mut change_project = false;
        let mut reload = false;
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(&proj.name).size(13.5).strong());
                let n = self.csv_rows.len();
                ui.label(
                    RichText::new(format!(
                        "#{} · {} · {} {}",
                        proj.id,
                        proj.container_type,
                        n,
                        i18n::plural(lang, n, "riga dati", "righe dati")
                    ))
                    .size(11.0)
                    .color(theme::muted(dark)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::action_button(
                        ui,
                        Icon::Folder,
                        i18n::t(lang, "Cambia progetto"),
                        ButtonKind::Neutral,
                    )
                    .clicked()
                    {
                        change_project = true;
                    }
                    if theme::action_button(
                        ui,
                        Icon::Refresh,
                        i18n::t(lang, "Ricarica"),
                        ButtonKind::Neutral,
                    )
                    .clicked()
                    {
                        reload = true;
                    }
                });
            });
        });

        if change_project {
            self.current_tab = SelectedTab::SearchList;
        }
        if reload {
            self.load_data_file();
            self.set_status(i18n::t(lang, "Dati ricaricati dal file."), false);
        }

        ui.add_space(10.0);

        if let Some(err) = &self.csv_load_error {
            theme::message_banner(ui, err, true);
            return;
        }
        if self.csv_headers.is_empty() {
            theme::card(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(i18n::t(
                        lang,
                        "Il file dati è vuoto o non è stato caricato correttamente.",
                    ))
                    .size(11.5)
                    .color(theme::muted(dark)),
                );
            });
            return;
        }

        theme::section_title(ui, i18n::t(lang, "Valori dei sensori"));
        ui.add_space(6.0);

        let headers = self.csv_headers.clone();
        let rows = self.csv_rows.clone();
        let text_height = 22.0;

        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::both()
                .max_height((ui.available_height() - 8.0).max(200.0))
                .show(ui, |ui| {
                    TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        // Scorrimento in entrambe le direzioni gestito dall'area
                        // esterna: disattivando lo scorrimento interno della tabella
                        // anche l'asse verticale resta visibile ai bordi della finestra.
                        .vscroll(false)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .columns(Column::auto().at_least(90.0), headers.len())
                        .header(26.0, |mut header| {
                            for (i, col_name) in headers.iter().enumerate() {
                                header.col(|ui| {
                                    let label = if col_name.trim().is_empty() {
                                        format!("{} {}", i18n::t(lang, "Colonna"), i + 1)
                                    } else {
                                        col_name.clone()
                                    };
                                    ui.label(
                                        RichText::new(label)
                                            .strong()
                                            .size(11.5)
                                            // Titoli in nero nel tema chiaro.
                                            .color(theme::text_strong(dark)),
                                    );
                                });
                            }
                        })
                        .body(|mut body| {
                            for (r, row) in rows.iter().enumerate() {
                                body.row(text_height, |mut row_ui| {
                                    for (c, cell) in row.iter().enumerate() {
                                        row_ui.col(|ui| {
                                            // Prima colonna (data/ora): testo in evidenza
                                            if c < 2 {
                                                ui.label(
                                                    RichText::new(cell)
                                                        .size(11.0)
                                                        .color(theme::muted(dark)),
                                                );
                                            } else {
                                                ui.label(RichText::new(cell).size(11.0));
                                            }
                                        });
                                    }
                                    let _ = r;
                                });
                            }
                        });
                });
        });
    }
}
