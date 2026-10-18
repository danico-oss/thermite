//! "Chart" tab: thermal chart with thresholds, toolbar, and PDF export.

use crate::data;
use crate::i18n;
use crate::theme::{self, ButtonKind, Icon};
use crate::views::ViewState;
use eframe::egui::{self, RichText};
use egui_plot::{CoordinatesFormatter, Corner, HLine, Legend, Line, LineStyle, Plot, PlotPoints, Points};

impl ViewState {
    pub fn show_chart_ui(&mut self, ui: &mut egui::Ui) {
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
                            "Apri un progetto dalla scheda Ricerca per vedere il grafico.",
                        ))
                        .size(11.0)
                        .color(theme::muted(dark)),
                    );
                    ui.add_space(16.0);
                });
            });
            return;
        };

        // --- Toolbar ---
        let mut export_clicked = false;
        let mut toggle_points = false;
        let mut toggle_legend = false;
        let mut reset_zoom = false;

        theme::card(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(RichText::new(&proj.name).size(13.5).strong());
                let badge = format!(
                    "{} {:.1} ÷ {:.1} °C",
                    i18n::t(lang, "soglie"),
                    proj.min_temp,
                    proj.max_temp
                );
                ui.label(RichText::new(badge).size(11.0).color(theme::muted(dark)));

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if theme::action_button(
                        ui,
                        Icon::Pdf,
                        i18n::t(lang, "Esporta PDF"),
                        ButtonKind::Primary,
                    )
                    .on_hover_text(i18n::t(lang, "Mostra l'anteprima del grafico in un PDF A4"))
                    .clicked()
                    {
                        export_clicked = true;
                    }
                    if theme::action_button(ui, Icon::Refresh, "Zoom", ButtonKind::Neutral)
                        .on_hover_text(i18n::t(lang, "Reimposta zoom e spostamento"))
                        .clicked()
                    {
                        reset_zoom = true;
                    }
                    if theme::pill_button(
                        ui,
                        Some(Icon::Legend),
                        i18n::t(lang, "Legenda"),
                        self.show_legend,
                    )
                    .clicked()
                    {
                        toggle_legend = true;
                    }
                    if theme::pill_button(
                        ui,
                        Some(Icon::Points),
                        i18n::t(lang, "Punti"),
                        self.show_points,
                    )
                    .clicked()
                    {
                        toggle_points = true;
                    }
                });
            });
        });

        if export_clicked {
            self.open_pdf_preview();
        }
        if reset_zoom {
            self.reset_zoom_requested = true;
        }
        if toggle_points {
            self.show_points = !self.show_points;
        }
        if toggle_legend {
            self.show_legend = !self.show_legend;
        }

        ui.add_space(10.0);

        if let Some(err) = &self.csv_load_error {
            theme::message_banner(ui, err, true);
            return;
        }

        if self.csv_rows.is_empty() || self.csv_headers.len() < 3 {
            theme::card(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(i18n::t(
                        lang,
                        "Dati insufficienti per il grafico: servono almeno le colonne Data, Ora e un sensore.",
                    ))
                    .size(11.5)
                    .color(theme::muted(dark)),
                );
            });
            return;
        }

        let series = data::parse_series(&self.csv_headers, &self.csv_rows);
        let Some(bounds) = data::bounds(&series, proj.min_temp, proj.max_temp) else {
            theme::card(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(
                    RichText::new(i18n::t(
                        lang,
                        "Nessun valore numerico valido trovato nelle colonne dei sensori.",
                    ))
                    .size(11.5)
                    .color(theme::WARNING),
                );
            });
            return;
        };

        let total = data::total_points(&series);
        let sensor_count = series.iter().filter(|s| !s.points.is_empty()).count();
        ui.horizontal(|ui| {
            theme::hint(
                ui,
                &format!(
                    "{} {} · {} {} · {} {} {} {}",
                    sensor_count,
                    i18n::plural(lang, sensor_count, "sensore", "sensori"),
                    total,
                    i18n::plural(lang, total, "campione", "campioni"),
                    i18n::t(lang, "da"),
                    data::format_timestamp(bounds.x_min),
                    i18n::t(lang, "a"),
                    data::format_timestamp(bounds.x_max)
                ),
            );
        });
        ui.add_space(6.0);

        // --- Chart ---
        let mut plot = Plot::new("temperature_line_points_plot")
            .height((ui.available_height() - 12.0).max(240.0))
            .allow_zoom(true)
            .allow_drag(true)
            .allow_scroll(true)
            .show_x(false)
            .show_y(false)
            .y_axis_label("°C")
            .coordinates_formatter(Corner::LeftTop, CoordinatesFormatter::new(|_, _| "".into()))
            .x_axis_formatter(|mark, _range| {
                let x = mark.value;
                if let Some(dt) = chrono::DateTime::from_timestamp(x as i64, 0) {
                    format!("{}\n{}", dt.format("%H:%M"), dt.format("%d/%m"))
                } else {
                    format!("{x:.0}")
                }
            })
            .y_axis_formatter(|mark, _| format!("{:.1} °C", mark.value));

        if self.show_legend {
            plot = plot.legend(Legend::default().position(Corner::RightTop));
        }
        if self.reset_zoom_requested {
            plot = plot.reset();
            self.reset_zoom_requested = false;
        }

        let tooltip_color = ui.visuals().text_color();
        plot.show(ui, |plot_ui| {
            for (i, serie) in series.iter().enumerate() {
                if serie.points.is_empty() {
                    continue;
                }
                let color = theme::series_color(i);
                let coords: Vec<[f64; 2]> = serie.points.clone();

                let line_pts: PlotPoints = coords.iter().copied().collect();
                plot_ui.line(Line::new(line_pts).name(&serie.name).color(color).width(1.8_f32));

                if self.show_points {
                    let points_pts: PlotPoints = coords.into_iter().collect();
                    plot_ui.points(
                        Points::new(points_pts)
                            .name(&serie.name)
                            .radius(2.5_f32)
                            .color(color),
                    );
                }
            }

            plot_ui.hline(
                HLine::new(proj.max_temp as f64)
                    .name(format!("{} {:.1} °C", i18n::t(lang, "Max"), proj.max_temp))
                    .color(theme::series_color(1))
                    .style(LineStyle::dashed_dense())
                    .width(1.4_f32),
            );
            plot_ui.hline(
                HLine::new(proj.min_temp as f64)
                    .name(format!("{} {:.1} °C", i18n::t(lang, "Min"), proj.min_temp))
                    .color(theme::series_color(0))
                    .style(LineStyle::dashed_dense())
                    .width(1.4_f32),
            );

            // --- Tooltip on the point closest to the cursor ---
            if let Some(pointer_pos) = plot_ui.pointer_coordinate() {
                let x_span = (bounds.x_max - bounds.x_min).max(1e-9);
                let y_span = (bounds.y_max - bounds.y_min).max(1e-9);
                let mut best: Option<(String, f64, String, String)> = None;
                let mut best_dist = f64::MAX;

                for serie in series.iter() {
                    for (idx, p) in serie.points.iter().enumerate() {
                        let dx = (p[0] - pointer_pos.x) / x_span;
                        let dy = (p[1] - pointer_pos.y) / y_span;
                        let dist = dx * dx + dy * dy;
                        if dist < best_dist {
                            best_dist = dist;
                            let (d, t) = serie
                                .stamps
                                .get(idx)
                                .cloned()
                                .unwrap_or_else(|| (String::new(), String::new()));
                            best = Some((serie.name.clone(), p[1], d, t));
                        }
                    }
                }

                if let Some((name, temp, d, t)) = best {
                    // threshold in normalized units: show only if the point is "close"
                    if best_dist < 0.02 {
                        let tooltip = format!(
                            "{name}\n{d} {t}\n{temp:.2} °C"
                        );
                        plot_ui.text(
                            egui_plot::Text::new(pointer_pos, tooltip).color(tooltip_color),
                        );
                    }
                }
            }
        });
    }
}
