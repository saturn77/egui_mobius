use egui::{CentralPanel, Context};
use egui_mobius_reactive::*;

use eframe::{App, Frame};

/// Demo application for testing ReactiveSlider
struct ReactiveSliderDemo {
    // Reactive values
    value1: Dynamic<f64>,
    value2: Dynamic<f64>,
    value3: Dynamic<f64>,
}

impl ReactiveSliderDemo {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            value1: Dynamic::new(50.0),
            value2: Dynamic::new(0.5),
            value3: Dynamic::new(25.0),
        }
    }
}

// Remove the #[derive(Default)] attribute

impl App for ReactiveSliderDemo {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("Reactive Slider Demo");
            ui.spacing();
            
            // Simple slider with default configuration
            ui.label("Basic slider:");
            ReactiveSlider::new(&self.value1)
                .with_range(0.0..=100.0)
                .show(ui);
            ui.label(format!("Value: {:.1}", self.value1.get()));
            ui.spacing();
            
            // Slider with display value and text
            ui.label("Slider with display value and label:");
            ReactiveSlider::new(&self.value2)
                .with_range(0.0..=1.0)
                .with_display_value(true)
                .with_text("Volume")
                .show(ui);
            ui.spacing();
            
            // Logarithmic slider
            ui.label("Logarithmic slider:");
            ReactiveSlider::new(&self.value3)
                .with_range(1.0..=100.0)
                .with_logarithmic(true)
                .with_display_value(true)
                .with_text("Sensitivity")
                .show(ui);
            ui.spacing();
            
            // Show values from all sliders
            ui.separator();
            ui.heading("Current Values");
            ui.label(format!("Value 1: {:.1}", self.value1.get()));
            ui.label(format!("Value 2: {:.3}", self.value2.get()));
            ui.label(format!("Value 3: {:.1}", self.value3.get()));
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Reactive Slider Demo",
        native_options,
        Box::new(|cc| Ok(Box::new(ReactiveSliderDemo::new(cc))))
    )
}