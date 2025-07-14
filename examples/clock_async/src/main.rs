use eframe::egui;
use egui_mobius::dispatching::AsyncDispatcher;
use egui_mobius::factory;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;
use std::time::Duration;
use egui_taffy::{taffy, tui};
use taffy::prelude::{length, percent, Style};
use egui_taffy::TuiBuilderLogic;
mod logger;
mod state;
mod types;
mod ui;

use state::AppState;
use types::{ClockMessage, Config, Event, Response};
use ui::{ControlPanel, LoggerPanel};

pub struct UiApp {
    state: AppState,
}

impl UiApp {
    pub fn new(state: AppState, mut response_slot: Slot<Response>) -> Self {
        let slider_ref = state.slider_value.clone();
        let combo_ref = state.combo_value.clone();
        let repaint = state.repaint.clone();

        response_slot.start(move |response| {
            match response {
                Response::SliderProcessed(val) => {
                    *slider_ref.lock().unwrap() = val;
                }
                Response::ComboProcessed(choice) => {
                    *combo_ref.lock().unwrap() = choice;
                }
            }
            repaint.request_repaint();
        });

        Self { state }
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Main layout using egui_taffy
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
                            flex_shrink: 1.0,
                            flex_basis: length(300.0),
                            padding: length(16.0),
                            ..Default::default()
                        })
                        .add(|tui| {
                            // Render the ControlPanel component
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
                            flex_shrink: 1.0,
                            padding: length(16.0),
                            size: taffy::Size {
                                width: length(800.0),
                                height: percent(100.0),
                            },
                            ..Default::default()
                        })
                        .add(|tui| {
                            // Render the LoggerPanel component
                            tui.ui(|ui| {
                                LoggerPanel::render(ui, &self.state);
                            });
                        });
                    });
            });
    }
}




fn background_generator_thread(clock_signal: Signal<ClockMessage>, _ctx: egui::Context) {
    std::thread::spawn(move || {
        loop {
            let now = chrono::Local::now();
            if let Err(e) = clock_signal.send(ClockMessage::TimeUpdated(now.format("%H:%M:%S").to_string())) {
                eprintln!("Failed to send TimeUpdated message: {e:?}");
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    });
}

fn main() {
    let config = {
        let local_dir = std::path::Path::new(".local");
        if !local_dir.exists() {
            let _ = std::fs::create_dir_all(local_dir);
        }
        let config_path = local_dir.join("config.json");
        
        if config_path.exists() {
            if let Ok(config_str) = std::fs::read_to_string(&config_path) {
                match serde_json::from_str(&config_str) {
                    Ok(config) => config,
                    Err(e) => {
                        eprintln!("Failed to parse config: {e}");
                        Config::default()
                    }
                }
            } else {
                eprintln!("Failed to read config file");
                Config::default()
            }
        } else {
            // Copy default config
            let default_config = Config::default();
            #[allow(clippy::collapsible_if)]
            if let Ok(json_data) = serde_json::to_string_pretty(&default_config) {
                if let Err(e) = std::fs::write(&config_path, json_data) {
                    eprintln!("Failed to write default config: {e}");
                }
            }
            default_config
        }
    };

    let (event_signal, event_slot) = factory::create_signal_slot::<Event>();
    let (response_signal, response_slot) = factory::create_signal_slot::<Response>();
    
    let dispatcher = AsyncDispatcher::new();
    dispatcher.attach_async(
        event_slot,
        response_signal.clone(),
        |event: Event| async move {
            match event {
                Event::SliderChanged(val) => {
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    Response::SliderProcessed(val)
                }
                Event::ComboSelected(choice) => {
                    tokio::time::sleep(Duration::from_millis(300)).await;
                    Response::ComboProcessed(choice)
                }
            }
        },
    );

    // Set up clock updates
    let (clock_signal, clock_slot) = factory::create_signal_slot::<ClockMessage>();
    let now = chrono::Local::now().format("%H:%M:%S").to_string();
    let _ = clock_signal.send(ClockMessage::TimeUpdated(now));

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_titlebar_buttons_shown(true)
            .with_min_inner_size([1150.0, 700.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Interactive Clock with Events",
        native_options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            
            // Start clock updates with UI context
            background_generator_thread(clock_signal, ctx.clone());

            // Create app state
            let app_state = AppState::new(ctx.clone(), config.clone());
            app_state.set_clock_slot(clock_slot);
            app_state.set_event_signal(event_signal.clone());
            
            Ok(Box::new(UiApp::new(app_state, response_slot)))
        }),
    ).expect("Failed to start application")
}
