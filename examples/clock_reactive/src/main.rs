mod state;
mod types;
mod ui;
mod runtime_integration;

use std::sync::Arc;
use eframe::egui;

use egui_taffy::{taffy, tui};
use taffy::prelude::{length, percent, Style};
use egui_taffy::TuiBuilderLogic;
use crate::state::AppState;
use crate::ui::{ControlPanel, LoggerPanel};
use crate::runtime_integration::RuntimeManager;

struct ClockApp {
    state    : Arc<AppState>,
    _runtime : RuntimeManager,
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

fn main() -> eframe::Result<()> {
    env_logger::init();
    // Create runtime with a multi-thread scheduler
    let rt = tokio::runtime::Runtime::new().unwrap();

    // Enter the runtime context
    let _guard = rt.enter();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0])
            .with_title("Clock Reactive Example"),
        ..Default::default()
    };

    // Store runtime in the app to keep it alive
    eframe::run_native(
        "Clock Reactive Example",
        options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();

            let config = std::fs::read_to_string("config.json")
                .ok()
                .and_then(|json| serde_json::from_str(&json).ok())
                .unwrap_or_default();

            let state = Arc::new(AppState::new(ctx.clone(), config));
            let mut runtime = RuntimeManager::new(state.clone());

            // Start the runtime from within the runtime context
            runtime.start(ctx);

            Ok(Box::new(ClockApp {
                state,
                _runtime: runtime,
            }))
        })
    )
}