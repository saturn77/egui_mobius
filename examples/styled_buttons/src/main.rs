
use eframe::egui::{CentralPanel, Context, Style, Visuals, Margin, Color32};
use egui_mobius_widgets::stateful_button::{StatefulButton, ButtonStyle};
use egui::{Ui, Widget, Response}; 

pub struct MyApp {
    button_start : StatefulButton,
    button_stop  : StatefulButton,
}

impl MyApp {
    pub fn new() -> Self {
        Self {
            button_start: StatefulButton::new(
                0,
                "Start Process",
                ButtonStyle {
                    stroke_size          : Some(2),
                    stroke_color         : Some(Color32::DARK_GREEN),
                    hovered_color        : Some(Color32::DARK_BLUE),
                    stroke_size_on_hover : Some(4), 
                    corner_radius        : Some(5),
                    inner_margin         : Some(Margin::same(8)),
                },
            ),
            button_stop: StatefulButton::new(
                0,
                "Stop Process",
                ButtonStyle {
                    stroke_size          : Some(2),
                    stroke_color         : Some(Color32::DARK_RED),
                    hovered_color        : Some(Color32::LIGHT_RED),
                    stroke_size_on_hover : Some(4), 
                    corner_radius        : Some(5),
                    inner_margin         : Some(Margin::same(4)),
                },
            ),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            let mut style = Style::default();
            style.visuals = Visuals::dark();
            // create a row layout for the buttons
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = (10.0, 0.0).into();
                self.button_start.ui_with_style(ui);
                self.button_stop.ui_with_style(ui);
            });

        });
    }
}

fn main() {
    let app = MyApp::new();
    let native_options = eframe::NativeOptions::default();
    
    if let Err(e) = eframe::run_native("My App", native_options, Box::new(|_cc| Ok(Box::new(app)))) {
        eprintln!("Failed to run the application: {}", e);
    }
}

