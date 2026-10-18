//! "Management" tab: creating / editing / deleting projects.

use crate::i18n;
use crate::theme::{self, ButtonKind, Icon};
use crate::views::ViewState;
use eframe::egui::{self, RichText, Vec2};

impl ViewState {
    pub fn show_management_ui(
        &mut self,
        ui: &mut egui::Ui,
        conn: &rusqlite::Connection,
        reload_projects: &mut bool,
    ) {
        let dark = ui.visuals().dark_mode;
        let lang = self.lang;
        let editing = self.selected_edit_id;

        // --- Page header ---
        ui.horizontal(|ui| {
            theme::section_title(
                ui,
                if editing.is_some() {
                    i18n::t(lang, "Modifica progetto")
                } else {
                    i18n::t(lang, "Nuovo progetto")
                },
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (text, color) = match editing {
                    Some(id) => (
                        format!("{} #{id}", i18n::t(lang, "ID univoco")),
                        theme::WARNING,
                    ),
                    None => (
                        i18n::t(lang, "ID assegnato automaticamente").to_string(),
                        theme::muted(dark),
                    ),
                };
                let text_w = ui
                    .fonts(|f| {
                        f.layout_no_wrap(text.clone(), egui::FontId::proportional(11.0), color)
                            .size()
                            .x
                    });
                let (rect, _) = ui.allocate_exact_size(
                    Vec2::new(text_w + 16.0, 19.0),
                    egui::Sense::hover(),
                );
                let painter = ui.painter();
                painter.rect_filled(
                    rect,
                    egui::CornerRadius::same(4),
                    color.gamma_multiply(0.14),
                );
                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(11.0),
                    color,
                );
            });
        });
        ui.add_space(6.0);

        // --- Form card ---
        let mut save_clicked = false;
        let mut delete_clicked = false;
        let mut cancel_clicked = false;

        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());

            egui::Grid::new("project_form")
                .num_columns(2)
                .spacing(Vec2::new(10.0, 7.0))
                .show(ui, |ui| {
                    theme::field_label(ui, i18n::t(lang, "Nome progetto"), 120.0);
                    ui.add_sized(
                        Vec2::new(300.0, 20.0),
                        egui::TextEdit::singleline(&mut self.name_input)
                            .hint_text(i18n::t(lang, "Es. Camera termostatica A1")),
                    );
                    ui.end_row();

                    theme::field_label(ui, i18n::t(lang, "Tipo di container"), 120.0);
                    ui.add_sized(
                        Vec2::new(300.0, 20.0),
                        egui::TextEdit::singleline(&mut self.container_input)
                            .hint_text(i18n::t(lang, "Es. Container refrigerato 20'")),
                    );
                    ui.end_row();

                    theme::field_label(ui, i18n::t(lang, "Temperature target (°C)"), 120.0);
                    ui.horizontal(|ui| {
                        ui.add_sized(
                            Vec2::new(95.0, 20.0),
                            egui::TextEdit::singleline(&mut self.min_temp_input)
                                .hint_text("min"),
                        );
                        ui.label("÷");
                        ui.add_sized(
                            Vec2::new(95.0, 20.0),
                            egui::TextEdit::singleline(&mut self.max_temp_input)
                                .hint_text("max"),
                        );
                        ui.add_space(6.0);
                        if self.min_temp_input.parse::<f32>().is_err()
                            || self.max_temp_input.parse::<f32>().is_err()
                        {
                            ui.label(
                                RichText::new(i18n::t(lang, "valori numerici richiesti"))
                                    .size(10.5)
                                    .color(theme::WARNING),
                            );
                        } else if let (Ok(min), Ok(max)) = (
                            self.min_temp_input.parse::<f32>(),
                            self.max_temp_input.parse::<f32>(),
                        ) {
                            if min > max {
                                ui.label(
                                    RichText::new(i18n::t(lang, "la minima supera la massima"))
                                        .size(10.5)
                                        .color(theme::DANGER),
                                );
                            }
                        }
                    });
                    ui.end_row();

                    theme::field_label(ui, i18n::t(lang, "File dati sensori"), 120.0);
                    ui.horizontal(|ui| {
                        let width = (ui.available_width() - 112.0).max(180.0);
                        ui.add_sized(
                            Vec2::new(width, 20.0),
                            egui::TextEdit::singleline(&mut self.file_path_input)
                                .hint_text(i18n::t(lang, "CSV, TXT oppure Excel (.xlsx / .xls)")),
                        );
                        if theme::action_button(
                            ui,
                            Icon::Folder,
                            i18n::t(lang, "Sfoglia…"),
                            ButtonKind::Neutral,
                        )
                        .clicked()
                        {
                            self.pick_data_file();
                        }
                    });
                    ui.end_row();
                });

            ui.add_space(6.0);
            theme::hint(
                ui,
                i18n::t(
                    lang,
                    "Formato atteso: prima colonna la data, seconda l'ora, colonne successive i sensori.",
                ),
            );

            // --- Outcome message ---
            if let Some((msg, is_err)) = self.feedback_msg.clone() {
                ui.add_space(6.0);
                theme::message_banner(ui, &msg, is_err);
            }

            // --- Actions ---
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if editing.is_none() {
                    if theme::action_button(
                        ui,
                        Icon::Save,
                        i18n::t(lang, "Salva progetto"),
                        ButtonKind::Primary,
                    )
                    .clicked()
                    {
                        save_clicked = true;
                    }
                } else {
                    if theme::action_button(
                        ui,
                        Icon::Save,
                        i18n::t(lang, "Salva modifiche"),
                        ButtonKind::Primary,
                    )
                    .clicked()
                    {
                        save_clicked = true;
                    }
                    if theme::action_button(
                        ui,
                        Icon::Trash,
                        i18n::t(lang, "Elimina"),
                        ButtonKind::Danger,
                    )
                    .clicked()
                    {
                        delete_clicked = true;
                    }
                }

                if editing.is_some()
                    && theme::action_button(
                        ui,
                        Icon::Close,
                        i18n::t(lang, "Annulla"),
                        ButtonKind::Neutral,
                    )
                    .clicked()
                {
                    cancel_clicked = true;
                }
            });
        });

        if save_clicked {
            self.try_save(conn, reload_projects);
        }
        if delete_clicked {
            self.try_delete(conn, reload_projects);
        }
        if cancel_clicked {
            self.clear_form();
            self.set_status(i18n::t(lang, "Modifica annullata."), false);
        }

        // --- Information card ---
        ui.add_space(6.0);
        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(
                RichText::new(i18n::t(lang, "Come si usa questa scheda"))
                    .size(12.0)
                    .strong(),
            );
            ui.add_space(2.0);
            for riga in [
                "1. Compila i campi e salva: il progetto finisce nel database SQLite.",
                "2. Collega il file CSV/Excel con i dati dei sensori e usa la scheda Ricerca per aprire griglia e grafico.",
                "3. Dalla scheda Ricerca puoi riaprire un progetto per modificarlo o eliminarlo.",
            ] {
                ui.label(
                    RichText::new(i18n::t(lang, riga))
                        .size(11.0)
                        .color(theme::muted(dark)),
                );
            }
        });
    }

    /// Validates the form fields and returns (min, max).
    fn validated_temps(&self) -> Result<(f32, f32), String> {
        let lang = self.lang;
        if self.name_input.trim().is_empty() {
            return Err(i18n::t(lang, "Il nome del progetto non può essere vuoto.").to_string());
        }
        let min = self
            .min_temp_input
            .parse::<f32>()
            .map_err(|_| {
                i18n::t(lang, "La temperatura minima deve essere un numero.").to_string()
            })?;
        let max = self
            .max_temp_input
            .parse::<f32>()
            .map_err(|_| {
                i18n::t(lang, "La temperatura massima deve essere un numero.").to_string()
            })?;
        if min > max {
            return Err(
                i18n::t(lang, "La temperatura minima non può superare la massima.").to_string(),
            );
        }
        Ok((min, max))
    }

    /// Saves the project (insert or update).
    pub fn try_save(&mut self, conn: &rusqlite::Connection, reload_projects: &mut bool) {
        let lang = self.lang;
        let (min, max) = match self.validated_temps() {
            Ok(v) => v,
            Err(e) => {
                self.feedback_msg = Some((e, true));
                return;
            }
        };

        // The data file's CRC32 is computed here, at save time, so that it
        // always stays consistent with the path being written.
        let crc = crate::crc32::file_hex(&self.file_path_input);

        let result = match self.selected_edit_id {
            Some(id) => conn.execute(
                "UPDATE projects SET name = ?1, container_type = ?2, min_temp = ?3, max_temp = ?4, data_file_path = ?5, data_crc32 = ?6 WHERE id = ?7",
                rusqlite::params![
                    &self.name_input,
                    &self.container_input,
                    min,
                    max,
                    &self.file_path_input,
                    crc,
                    id
                ],
            ),
            None => conn.execute(
                "INSERT INTO projects (name, container_type, min_temp, max_temp, data_file_path, data_crc32) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![
                    &self.name_input,
                    &self.container_input,
                    min,
                    max,
                    &self.file_path_input,
                    crc
                ],
            ),
        };

        match result {
            Ok(_) => {
                let msg = match self.selected_edit_id {
                    Some(id) => format!(
                        "{} #{id} {}",
                        i18n::t(lang, "Progetto"),
                        i18n::t(lang, "aggiornato con successo!")
                    ),
                    None => format!(
                        "{} {}",
                        i18n::t(lang, "Progetto"),
                        i18n::t(lang, "salvato con successo!")
                    ),
                };
                self.feedback_msg = Some((msg.clone(), false));
                self.set_status(msg, false);
                self.clear_form();
                *reload_projects = true;
            }
            Err(e) => {
                self.feedback_msg = Some((
                    format!("{} {e}", i18n::t(lang, "Errore SQLite:")),
                    true,
                ));
            }
        }
    }

    /// Deletes the project currently being edited.
    pub fn try_delete(&mut self, conn: &rusqlite::Connection, reload_projects: &mut bool) {
        let lang = self.lang;
        let Some(id) = self.selected_edit_id else {
            self.feedback_msg = Some((
                i18n::t(
                    lang,
                    "Seleziona un progetto dalla scheda Ricerca prima di eliminarlo.",
                )
                .to_string(),
                true,
            ));
            self.set_status(
                i18n::t(lang, "Nessun progetto selezionato da eliminare."),
                true,
            );
            return;
        };

        match conn.execute("DELETE FROM projects WHERE id = ?1", [id]) {
            Ok(_) => {
                let msg = format!(
                    "{} #{id} {}",
                    i18n::t(lang, "Progetto"),
                    i18n::t(lang, "eliminato.")
                );
                self.feedback_msg = Some((msg.clone(), false));
                self.set_status(msg, false);
                self.clear_form();
                *reload_projects = true;
            }
            Err(e) => {
                self.feedback_msg = Some((
                    format!("{} {e}", i18n::t(lang, "Errore delete:")),
                    true,
                ));
            }
        }
    }
}
