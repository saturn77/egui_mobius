mod state;
mod types;
mod ui;

use eframe::egui;
use std::time::Duration;

use types::ClockMessage;
use egui_taffy::{taffy, tui};
use taffy::prelude::{length, percent, Style};
use egui_taffy::TuiBuilderLogic;
use egui_mobius::Signal;

use crate::state::AppState;
use crate::ui::{ControlPanel, LoggerPanel};

struct ClockApp {
    state: AppState,
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            tui(ui, "main_panel")
                .reserve_available_space()
                .style(taffy::Style {
                    display: taffy::Display::Flex,
                    flex_direction: taffy::FlexDirection::Row,
                    align_items: Some(taffy::AlignItems::Stretch),
                    padding: length(4.),
                    gap: length(8.),
                    size: taffy::Size {
                        width: percent(1.0),
                        height: percent(1.0),
                    },
                    ..Default::default()
                })
                .show(|tui| {
                    // Left column (controls)
                    tui.style(Style {
                        display: taffy::Display::Flex,
                        flex_direction: taffy::FlexDirection::Column,
                        gap: length(8.0),
                        flex_grow: 0.0,
                        flex_shrink: 0.0,
                        flex_basis: length(300.0),
                        padding: length(16.0),
                        ..Default::default()
                    })
                    .add(|tui| {
                        tui.ui(|ui| {
                            ControlPanel::render(ui, &self.state);
                        });
                    });

                    // Right column (logger)
                    tui.style(Style {
                        display: taffy::Display::Flex,
                        flex_direction: taffy::FlexDirection::Column,
                        gap: length(8.0),
                        flex_grow: 1.0,
                        flex_shrink: 0.0,
                        padding: length(16.0),
                        size: taffy::Size {
                            width: length(800.0),
                            height: percent(100.0),
                        },
                        ..Default::default()
                    })
                    .add(|tui| {
                        tui.ui(|ui| {
                            LoggerPanel::render(ui, &self.state);
                        });
                    });
                });
        });
    }
}

fn background_generator_thread(clock_signal: Signal<ClockMessage>, ctx: egui::Context) {
    std::thread::spawn(move || {
        loop {
            if let Err(e) = clock_signal.send(ClockMessage::TimeUpdated(())) {
                eprintln!("Failed to send TimeUpdated message: {:?}", e);
            }
            std::thread::sleep(Duration::from_secs(1));
            ctx.request_repaint();
        }
    });
}


fn main() -> eframe::Result<()> {
    // Create signal and slot for clock updates
    let (clock_signal, clock_slot) = egui_mobius::factory::create_signal_slot::<ClockMessage>();
    
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("Clock Reactive Example"),
        ..Default::default()
    };

    eframe::run_native(
        "Clock Reactive Example",
        options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            
            // Start clock updates with UI context
            background_generator_thread(clock_signal.clone(), ctx.clone());

            // Load config from file or use defaults
            let config = std::fs::read_to_string("config.json")
                .ok()
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();

            // Create app state
            let state = AppState::new(ctx.clone(), config);
            state.set_clock_slot(clock_slot);

            Ok(Box::new(ClockApp { state }))
        })
    )
}