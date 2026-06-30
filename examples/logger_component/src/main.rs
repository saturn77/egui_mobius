//! Custom Log Types Example — port of `egui_lens/examples/basic_custom/`.
//!
//! Demonstrates lens's full feature surface in the workspace context:
//! reactive shared state via `Dynamic<ReactiveEventLoggerState>`, custom
//! per-type colors via `Dynamic<LogColors>`, named custom log types
//! (network, database, security, etc.), system-info logging, and the
//! `with_colors` constructor for color-aware logger views.

use eframe::egui;
use egui_lens::{LogColors, ReactiveEventLogger, ReactiveEventLoggerState};
use egui_mobius_reactive::Dynamic;

mod platform;
use platform::{banner, details, parameters::gui};

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_titlebar_buttons_shown(true)
            .with_inner_size([gui::VIEWPORT_X * 1.25, gui::VIEWPORT_Y * 1.25])
            .with_min_inner_size([gui::VIEWPORT_X, gui::VIEWPORT_Y])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Custom Log Types Example",
        native_options,
        Box::new(|cc| Ok(Box::new(ExampleApp::new(cc)))),
    )
}

struct ExampleApp {
    logger_state: Dynamic<ReactiveEventLoggerState>,
    log_colors: Dynamic<LogColors>,
    banner: banner::Banner,
    details: details::Details,
}

impl ExampleApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let logger_state = Dynamic::new(ReactiveEventLoggerState::new());

        let mut log_colors = Dynamic::new(LogColors::default());
        configure_custom_log_colors(&mut log_colors);

        let mut banner = banner::Banner::new();
        let mut details = details::Details::new();

        banner.format();
        details.get_os();

        let app = Self {
            logger_state,
            log_colors,
            banner,
            details,
        };

        app.add_example_logs();

        app
    }

    fn add_example_logs(&self) {
        let logger = ReactiveEventLogger::with_colors(&self.logger_state, &self.log_colors);

        logger.log_info(&self.banner.message);

        let details_text = self.details.clone().format_os();
        logger.log_info(&details_text);

        logger.log_info("This is a standard info message");
        logger.log_warning("This is a standard warning message");
        logger.log_error("This is a standard error message");
        logger.log_debug("This is a standard debug message");

        logger.log_custom("network", "Connected to server on port 8080");
        logger.log_custom("database", "Executed query in 42ms");
        logger.log_custom("security", "User authentication successful");
        logger.log_custom("performance", "Rendering took 16ms");
        logger.log_custom("analytics", "Page view recorded for /dashboard");
        logger.log_custom("http", "GET /api/users - 200 OK - 12ms");
        logger.log_custom("websocket", "Client connected: user_123");
        logger.log_custom("auth", "JWT token issued");
    }
}

impl eframe::App for ExampleApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Honor a "show system info" request stashed in egui memory by the
        // logger's UI (matches basic_custom's behavior).
        let show_system_info = ctx.memory(|mem| {
            mem.data
                .get_temp::<bool>(egui::Id::new("show_system_info"))
                .unwrap_or(false)
        });

        if show_system_info {
            ctx.memory_mut(|mem| {
                mem.data.remove::<bool>(egui::Id::new("show_system_info"));
            });

            let logger = ReactiveEventLogger::with_colors(&self.logger_state, &self.log_colors);
            let details_text = self.details.format_os();
            logger.log_info(&details_text);
            logger.log_info(&self.banner.message);
        }

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Custom Log Types Example");
            ui.add_space(8.0);

            ui.label("This example demonstrates the flexible custom log types feature.");
            ui.label("Each custom log type has its own specific color and identifier.");
            ui.add_space(16.0);

            let logger = ReactiveEventLogger::with_colors(&self.logger_state, &self.log_colors);
            logger.show(ui);

            ui.add_space(16.0);
            ui.heading("Add more logs");

            ui.horizontal(|ui| {
                if ui.button("System Info").clicked() {
                    let details_text = self.details.format_os();
                    logger.log_info(&details_text);
                }
                if ui.button("Add Network Log").clicked() {
                    logger.log_custom("network", "New client connected from 192.168.1.5");
                }
                if ui.button("Add Database Log").clicked() {
                    logger.log_custom("database", "Inserted 5 records in 18ms");
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Add Security Log").clicked() {
                    logger.log_custom("security", "Failed login attempt: incorrect password");
                }
                if ui.button("Add Custom HTTP Log").clicked() {
                    logger.log_custom("http", "POST /api/data - 201 Created - 45ms");
                }
                if ui.button("Add Standard Info Log").clicked() {
                    logger.log_info("This is a standard info message");
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Add Standard Warning").clicked() {
                    logger.log_warning("This is a standard warning message");
                }
                if ui.button("Add Standard Error").clicked() {
                    logger.log_error("This is a standard error message");
                }
                if ui.button("Add Standard Debug").clicked() {
                    logger.log_debug("This is a standard debug message");
                }
            });
        });
    }
}

fn configure_custom_log_colors(colors: &mut Dynamic<LogColors>) {
    let mut colors_value = colors.get();

    colors_value.set_custom_color("network", egui::Color32::from_rgb(100, 149, 237));
    colors_value.set_custom_color("database", egui::Color32::from_rgb(106, 90, 205));
    colors_value.set_custom_color("security", egui::Color32::from_rgb(60, 179, 113));

    colors_value.set_custom_colors(
        "performance",
        egui::Color32::from_rgb(255, 165, 0),
        egui::Color32::from_rgb(255, 215, 140),
    );

    colors_value.set_custom_colors(
        "analytics",
        egui::Color32::from_rgb(218, 112, 214),
        egui::Color32::from_rgb(230, 175, 228),
    );

    colors_value.set_custom_colors(
        "http",
        egui::Color32::from_rgb(70, 130, 180),
        egui::Color32::from_rgb(150, 190, 220),
    );

    colors_value.set_custom_colors(
        "websocket",
        egui::Color32::from_rgb(0, 139, 139),
        egui::Color32::from_rgb(100, 200, 200),
    );

    colors_value.set_custom_colors(
        "auth",
        egui::Color32::from_rgb(85, 107, 47),
        egui::Color32::from_rgb(160, 200, 120),
    );

    colors.set(colors_value);
}
