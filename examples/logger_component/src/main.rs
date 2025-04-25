use eframe::egui;
use egui_mobius_components::*;
use egui_mobius_components::components::event_logger::processor::run_logger_backend;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_titlebar_buttons_shown(true)
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Event Logger Example",
        native_options,
        Box::new(|cc| {
            // Initialize logger with signal/slot
            let (logger, event_slot, response_signal) = create_event_logger(
                cc.egui_ctx.clone(), 
                LogColors::default()
            );
            
            // Run the logger backend
            run_logger_backend(event_slot, response_signal);
            
            // Create the app with the logger
            Ok(Box::new(MyApp::new(logger)))
        }),
    )
}

struct MyApp {
    logger: EguiMobiusEventLogger,
    counter: i32,
}

impl MyApp {
    fn new(logger: EguiMobiusEventLogger) -> Self {
        // Add a welcome message
        logger.info(
            "Application started".to_string(),
            LogSender::system(),
            LogType::Default
        );

        Self {
            logger,
            counter: 0,
        }
    }
    
    fn add_random_log(&mut self) {
        self.counter += 1;
        
        // Create different message types based on counter
        let message = match self.counter % 4 {
            0 => Message::Info(format!("Info message #{}", self.counter)),
            1 => Message::Warn(format!("Warning message #{}", self.counter)),
            2 => Message::Debug(format!("Debug message #{}", self.counter)),
            _ => Message::Error(format!("Error message #{}", self.counter)),
        };

        // Create different sender types based on counter
        let sender = match self.counter % 5 {
            0 => LogSender::button(format!("button_{}", self.counter)),
            1 => LogSender::slider(format!("slider_{}", self.counter)),
            2 => LogSender::checkbox(format!("checkbox_{}", self.counter)),
            3 => LogSender::text_field(format!("text_field_{}", self.counter)),
            _ => LogSender::custom(format!("custom_widget_{}", self.counter)),
        };

        // Create different log styles based on counter
        let style = match self.counter % 6 {
            0 => LogType::Default,
            1 => LogType::Slider,
            2 => LogType::OptionA,
            3 => LogType::OptionB,
            4 => LogType::CustomEvent,
            _ => LogType::RunStop,
        };

        // Add the log entry
        self.logger.add_log(message, sender, style);
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Event Logger Example");
                
                // Button row
                ui.horizontal(|ui| {
                    if ui.button("Add Random Log").clicked() {
                        self.add_random_log();
                    }
                    
                    if ui.button("Add Info Log").clicked() {
                        self.counter += 1;
                        self.logger.info(
                            format!("Information log at time {}", self.counter),
                            LogSender::button("Info Button"),
                            LogType::Default
                        );
                    }
                    
                    if ui.button("Add Warning Log").clicked() {
                        self.counter += 1;
                        self.logger.warn(
                            format!("Warning log at time {}", self.counter),
                            LogSender::slider("Warning Slider"),
                            LogType::Slider
                        );
                    }
                    
                    if ui.button("Add Error Log").clicked() {
                        self.counter += 1;
                        self.logger.error(
                            format!("Error log at time {}", self.counter),
                            LogSender::text_field("Error Text Field"),
                            LogType::RunStop
                        );
                    }
                    
                    if ui.button("Clear Log").clicked() {
                        self.logger.clear();
                    }
                    
                    // Background thread button
                    if ui.button("Add Logs From Thread").clicked() {
                        let logger = self.logger.clone();
                        thread::spawn(move || {
                            for i in 0..5 {
                                logger.info(
                                    format!("Background thread log #{}", i),
                                    LogSender::button("Background Thread"),
                                    LogType::Primary
                                );
                                thread::sleep(Duration::from_millis(500));
                            }
                        });
                    }
                });
                
                ui.separator();
                
                // Display the logger
                ui.heading("Event Log");
                ui.separator();
                
                // Create a scrollable area for the logger
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.logger.show(ui);
                });
            });
        });
    }
}