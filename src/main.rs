mod crc32;
mod data;
mod export;
mod i18n;
mod icon;
mod models;
mod scene3d;
mod settings;
mod theme;
mod views;

use eframe::egui;
use i18n::{t, Language};
use models::{Command, Database, Project, SelectedTab};
use views::ViewState;

struct ProjectApp {
    db: Database,
    view_state: ViewState,
    projects: Vec<Project>,
    theme_applied: bool,
}

impl ProjectApp {
    fn new(lang: Language, dark_mode: bool) -> Self {
        let db = Database::new();
        let projects = db.load_all_projects();
        let mut view_state = ViewState::new();
        view_state.lang = lang;
        view_state.dark_mode = dark_mode;
        view_state.clear_form();

        Self {
            db,
            view_state,
            projects,
            theme_applied: false,
        }
    }

    fn reload_projects(&mut self) {
        self.projects = self.db.load_all_projects();
    }

    fn handle_command(&mut self, cmd: Command, ctx: &egui::Context) {
        let lang = self.view_state.lang;
        let mut reload = false;
        match cmd {
            Command::NewProject => {
                self.view_state.current_tab = SelectedTab::Management;
                self.view_state.clear_form();
                self.view_state.set_status(
                    t(lang, "Nuovo progetto: compila i campi e salva."),
                    false,
                );
            }
            Command::SaveProject => {
                self.view_state.current_tab = SelectedTab::Management;
                self.view_state.try_save(&self.db.conn, &mut reload);
            }
            Command::DeleteProject => {
                self.view_state.current_tab = SelectedTab::Management;
                self.view_state.try_delete(&self.db.conn, &mut reload);
            }
            Command::ReloadProjects => {
                self.reload_projects();
                let n = self.projects.len();
                self.view_state.set_status(
                    format!("{} {n} {}.", t(lang, "Elenco aggiornato:"), t(lang, "progetti")),
                    false,
                );
            }
            Command::PickDataFile => {
                self.view_state.pick_data_file();
                self.view_state
                    .set_status(t(lang, "Percorso del file dati aggiornato."), false);
            }
            Command::ReloadData => {
                self.view_state.load_data_file();
                match &self.view_state.csv_load_error {
                    Some(e) => self.view_state.set_status(format!("{} {e}", t(lang, "Errore:")), true),
                    None => {
                        let n = self.view_state.csv_rows.len();
                        let noun = i18n::plural(lang, n, "riga dati", "righe dati");
                        self.view_state.set_status(
                            format!("{} {n} {noun}.", t(lang, "Dati ricaricati:")),
                            false,
                        );
                    }
                }
            }
            Command::SwitchTab(tab) => {
                self.view_state.current_tab = tab;
                if matches!(
                    tab,
                    SelectedTab::DataGrid | SelectedTab::ChartPlot | SelectedTab::SensorMap
                ) && self.view_state.active_project.is_some()
                {
                    self.view_state.load_data_file();
                }
            }
            Command::ToggleTheme => {
                self.view_state.dark_mode = !self.view_state.dark_mode;
                theme::apply(ctx, self.view_state.dark_mode);
                settings::save(&settings::Settings {
                    dark_mode: self.view_state.dark_mode,
                    lang: self.view_state.lang,
                });
                self.view_state.set_status(
                    if self.view_state.dark_mode {
                        t(lang, "Tema scuro attivo.")
                    } else {
                        t(lang, "Tema chiaro attivo.")
                    },
                    false,
                );
            }
            Command::OpenSettings => {
                self.view_state.show_settings = true;
            }
            Command::ExportPdf => {
                self.view_state.open_pdf_preview();
            }
            Command::ExportMapPdf => {
                self.view_state.open_map_preview();
            }
            Command::ResetMapView => {
                self.view_state.reset_map_view();
            }
            Command::ClearSearch => {
                self.view_state.search_query.clear();
                self.view_state.set_status(t(lang, "Ricerca azzerata."), false);
            }
            Command::TogglePoints => {
                self.view_state.show_points = !self.view_state.show_points;
            }
            Command::ToggleLegend => {
                self.view_state.show_legend = !self.view_state.show_legend;
            }
            Command::ResetZoom => {
                self.view_state.reset_zoom_requested = true;
            }
            Command::About => {
                self.view_state.show_about = true;
            }
            Command::Quit => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        if reload {
            self.reload_projects();
        }
    }
}

impl eframe::App for ProjectApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Compact scale, also suited to laptop screens.
        ctx.set_pixels_per_point(1.0);

        if !self.theme_applied {
            theme::apply(ctx, self.view_state.dark_mode);
            self.theme_applied = true;
        }

        let dark_mode = self.view_state.dark_mode;
        let mut cmd: Option<Command> = None;

        // --- Menu bar + ribbon bar ---
        // The background follows the theme: light in light theme, dark in dark theme.
        let top_frame = egui::Frame::new()
            .fill(theme::ribbon_bg(dark_mode))
            .inner_margin(egui::Margin {
                left: 6,
                right: 6,
                top: 3,
                bottom: 0,
            });
        egui::TopBottomPanel::top("top_panel")
            .frame(top_frame)
            .show(ctx, |ui| {
                if let Some(c) = views::ribbon::show_top_bar(ui, &self.view_state) {
                    cmd = Some(c);
                }
            });

        // --- Status bar ---
        let bottom_frame = egui::Frame::new()
            .fill(theme::ribbon_bg(dark_mode))
            .inner_margin(egui::Margin {
                left: 8,
                right: 8,
                top: 2,
                bottom: 2,
            });
        egui::TopBottomPanel::bottom("status_bar")
            .frame(bottom_frame)
            .show(ctx, |ui| {
                views::ribbon::show_status_bar(ui, &self.view_state, self.projects.len());
            });

        // --- Central area ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(4.0);

            let mut reload_needed = false;
            match self.view_state.current_tab {
                SelectedTab::Management => {
                    self.view_state
                        .show_management_ui(ui, &self.db.conn, &mut reload_needed);
                }
                SelectedTab::SearchList => {
                    self.view_state.show_search_ui(ui, &self.projects);
                }
                SelectedTab::DataGrid => {
                    self.view_state.show_datagrid_ui(ui);
                }
                SelectedTab::ChartPlot => {
                    self.view_state.show_chart_ui(ui);
                }
                SelectedTab::SensorMap => {
                    self.view_state.show_map_ui(ui, &self.db);
                }
            }

            if reload_needed {
                self.projects = self.db.load_all_projects();
            }
        });

        if let Some(c) = cmd {
            self.handle_command(c, ctx);
        }

        views::ribbon::show_about_window(ctx, &mut self.view_state);
        self.view_state.show_settings_window(ctx);
        self.view_state.show_pdf_preview_window(ctx);
        self.view_state.show_map_preview_window(ctx);
    }

    /// Window background color: follows the theme, so that any transparent
    /// areas do not show eframe's default dark gray.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme::panel_bg(self.view_state.dark_mode).to_normalized_gamma_f32()
    }
}

fn main() -> eframe::Result<()> {
    let user_settings = settings::load();
    let title = format!(
        "Thermite — {}",
        t(
            user_settings.lang,
            "Mappatura Termica di Trasporto"
        )
    );

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size((900.0, 600.0))
            .with_min_inner_size((680.0, 460.0))
            .with_icon(icon::egui_icon(64))
            .with_title(&title),
        ..eframe::NativeOptions::default()
    };

    eframe::run_native(
        &title,
        native_options,
        Box::new(move |_| {
            Ok(Box::new(ProjectApp::new(
                user_settings.lang,
                user_settings.dark_mode,
            )))
        }),
    )
}
