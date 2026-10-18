//! "Settings" window: choice of the interface language and theme.

use crate::i18n::{self, Language};
use crate::settings;
use crate::theme::{self, ButtonKind, Icon};
use crate::views::ViewState;
use eframe::egui::{self, Align2, RichText, Vec2};

impl ViewState {
    /// Draws (if open) the configurations window and immediately applies
    /// the language and theme changes, saving them to disk.
    pub fn show_settings_window(&mut self, ctx: &egui::Context) {
        if !self.show_settings {
            return;
        }

        let lang = self.lang;
        let mut new_lang = self.lang;
        let mut new_dark = self.dark_mode;
        let mut open = true;
        let mut close_clicked = false;

        egui::Window::new(i18n::t(lang, "Configurazioni"))
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                ui.set_min_width(340.0);

                // --- Language ---
                theme::section_title(ui, i18n::t(lang, "Lingua"));
                theme::hint(ui, i18n::t(lang, "Scegli la lingua dell'interfaccia."));
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    for l in Language::ALL {
                        let selected = l == lang;
                        if theme::pill_button(ui, None, l.label(), selected).clicked() {
                            new_lang = l;
                        }
                    }
                });

                ui.add_space(12.0);

                // --- Theme ---
                theme::section_title(ui, i18n::t(lang, "Tema"));
                theme::hint(
                    ui,
                    i18n::t(lang, "Il tema scuro riduce l'affaticamento visivo."),
                );
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if theme::pill_button(ui, Some(Icon::Sun), i18n::t(lang, "Chiaro"), !new_dark)
                        .clicked()
                    {
                        new_dark = false;
                    }
                    if theme::pill_button(ui, Some(Icon::Moon), i18n::t(lang, "Scuro"), new_dark)
                        .clicked()
                    {
                        new_dark = true;
                    }
                });

                ui.add_space(10.0);
                theme::hint(
                    ui,
                    i18n::t(lang, "Le modifiche vengono salvate automaticamente."),
                );

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    // Preview of the text in the chosen language (so the user
                    // immediately sees the effect of the selection).
                    if theme::action_button(
                        ui,
                        Icon::Close,
                        i18n::t(lang, "Chiudi"),
                        ButtonKind::Neutral,
                    )
                    .clicked()
                    {
                        close_clicked = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new("Thermite")
                                .size(11.0)
                                .color(theme::muted(new_dark)),
                        );
                    });
                });
            });

        if new_lang != self.lang || new_dark != self.dark_mode {
            self.lang = new_lang;
            self.dark_mode = new_dark;
            theme::apply(ctx, self.dark_mode);
            settings::save(&settings::Settings {
                dark_mode: self.dark_mode,
                lang: self.lang,
            });
        }

        if close_clicked || !open {
            self.show_settings = false;
        }
    }
}
