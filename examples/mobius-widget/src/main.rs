// src/main.rs
use eframe::egui;
use std::sync::Arc;
use egui_mobius_reactive::*;

mod widgets; 
use widgets::{counter::CounterWidget, logger::LoggerWidget};

struct MyApp {
    counter_widget: Arc<dyn MobiusWidget>,
    logger_widget: Arc<dyn MobiusWidget>,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("This is a counter widget:");
            self.counter_widget.render_widget(ui);
            ui.add_space(16.0);
            ui.label("This is a textarea widget:");
            self.logger_widget.render_widget(ui);
            ui.add_space(16.0);
        });
    }
}

fn main() -> eframe::Result<()> {
    let count = Dynamic::new(0);
    
    let log_text = Dynamic::new("Ready.".to_string());

    let mut counter = CounterWidget::default();
    
    // Instantiate a new counter widget, with multiple "dynamic" function signature arguments.
    // This is a simple counter that can be incremented and decremented.
    // The counter widget will log its state to the logger widget.

    counter.with_dynamic(Arc::new(count.clone()));
    counter.with_dynamic(Arc::new(log_text.clone()));

    // The logger widget will display the log messages.
    let mut logger = LoggerWidget::default();
    logger.with_dynamic(Arc::new(log_text));

    let app = MyApp {
        counter_widget: Arc::new(counter),
        logger_widget: Arc::new(logger),
    };

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Mobius Reactive Widgets Demo", 
        native_options, 
        Box::new(|_cc| Ok(Box::new(app)))
    )
}
