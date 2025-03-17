use chrono::Local;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_mobius::dispatching::AsyncDispatcher;
use egui_mobius::factory;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;
use std::time::Duration;

mod logger;
mod state;
mod types;

use logger::{LogColors, LogEntry};
use state::AppState;
use types::{ClockMessage, Config, Event, Response};


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
        egui::SidePanel::right("log_panel")
            .resizable(true)
            .default_width(700.0)
            .min_width(500.0)
            .max_width(900.0)
            .show(ctx, |ui| {
                let total_events = self.state.logs.lock().unwrap().len();
                ui.heading(format!("Event Log ({} events)", total_events));

                let log_filters = self.state.log_filters.lock().unwrap().clone();
                ui.horizontal(|ui| {
                    ui.label("Source filter:");
                    for source in [&"ui", &"clock"] {
                        let source = source.to_string();
                        let selected = log_filters.contains(&source);
                        if ui.selectable_label(selected, &source).clicked() {
                            let mut filters = self.state.log_filters.lock().unwrap();
                            if selected {
                                filters.retain(|s| s != &source);
                            } else {
                                filters.push(source.clone());
                            }
                        }
                    }
                });

                if ui.button("Clear Logger").clicked() {
                    self.state.logs.lock().unwrap().clear();
                }

                // Get all logs and filters at once to avoid multiple locks
                let logs = self.state.logs.lock().unwrap().clone();
                let filters = self.state.log_filters.lock().unwrap().clone();
                let colors = self.state.colors.lock().unwrap().clone();

                // Event counter in header
                let total_events = logs.len();
                ui.heading(format!("Event Log ({} events)", total_events));
                ui.add_space(8.0);

                // Create a table for the log entries
                egui::ScrollArea::vertical().id_salt("log_scroll").show(ui, |ui| {
                    let table = TableBuilder::new(ui)
                        .striped(true)
                        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                        .column(Column::exact(280.0))
                        .column(Column::exact(20.0))  // Spacer column
                        .column(Column::exact(400.0))
                        .header(20.0, |mut header| {
                            header.col(|ui| {
                                ui.label(egui::RichText::new("Time Updates").strong().monospace());
                            });
                            header.col(|_| {}); // Empty spacer column
                            header.col(|ui| {
                                ui.label(egui::RichText::new("UI Events").strong().monospace());
                            });
                        });

                    let mut time_updates = Vec::new();
                    let mut ui_events = Vec::new();

                    // Split entries into two columns
                    for entry in logs.iter().rev() {
                        if !filters.contains(&entry.source) {
                            continue;
                        }
                        if entry.source == "clock" {
                            time_updates.push(entry);
                        } else if entry.source == "ui" {
                            ui_events.push(entry);
                        }
                    }

                    // Display entries side by side
                    let max_entries = time_updates.len().max(ui_events.len());
                    table.body(|mut body| {
                        for i in 0..max_entries {
                            body.row(18.0, |mut row| {
                                // Time Updates column
                                row.col(|ui| {
                                    if let Some(entry) = time_updates.get(i) {
                                        let text = egui::RichText::new(entry.formatted()).monospace();
                                        ui.label(text.color(colors.clock));
                                    }
                                });

                                // Spacer column
                                row.col(|_| {});

                                // UI Events column
                                row.col(|ui| {
                                    if let Some(entry) = ui_events.get(i) {
                                        let color = if entry.message.contains("Slider value") {
                                            colors.slider
                                        } else if entry.message.contains("Selected option: Option A") {
                                            colors.option_a
                                        } else if entry.message.contains("Selected option: Option B") {
                                            colors.option_b
                                        } else if entry.message.contains("Selected option: Option C") {
                                            colors.option_c
                                        } else if entry.message.contains("Time format changed") {
                                            colors.time_format
                                        } else if entry.message.contains("Custom Event") {
                                            colors.custom_event
                                        } else {
                                            colors.time_format // Default color
                                        };
                                        let text = egui::RichText::new(entry.formatted()).monospace();
                                        ui.label(text.color(color));
                                    }
                                });
                            });
                        }
                    });
                });
            });
    
        egui::CentralPanel::default().show(ctx, |ui| {
            let time = self.state.current_time.lock().unwrap().clone();
            ui.heading("🕒 Live Clock");
            ui.horizontal(|ui| {
                ui.add(egui::Label::new(egui::RichText::new(format!("Current Time: {}", time)).size(24.0)));
            });
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Time Format:");
                    ui.add_space(10.0);
                    let mut use_24h = *self.state.use_24h.lock().unwrap();
                    if ui.selectable_label(use_24h, "24h").clicked() {
                        use_24h = true;
                        *self.state.use_24h.lock().unwrap() = use_24h;
                        self.state.log("ui", "Time format changed to 24-hour".to_string());
                        self.state.save_config();
                    }
                    if ui.selectable_label(!use_24h, "12h").clicked() {
                        use_24h = false;
                        *self.state.use_24h.lock().unwrap() = use_24h;
                        self.state.log("ui", "Time format changed to 12-hour".to_string());
                        self.state.save_config();
                    }
                });
            });

            ui.add_space(20.0);
            ui.group(|ui| {
                ui.heading("Controls");
                ui.add_space(10.0);

                // Slider
                let mut slider_value = *self.state.slider_value.lock().unwrap();
                if ui.add(egui::Slider::new(&mut slider_value, 0.0..=100.0).text("Value")).changed() {
                    *self.state.slider_value.lock().unwrap() = slider_value;
                    if let Some(signal) = &*self.state.event_signal.lock().unwrap() {
                        let _ = signal.send(Event::SliderChanged(slider_value));
                    }
                    self.state.log("ui", format!("Slider value changed to {}", slider_value));
                    self.state.save_config();
                }

                // Combo Box
                ui.vertical(|ui| {
                    ui.label("Select Option:");
                    let mut combo_value = self.state.combo_value.lock().unwrap().clone();
                    for option in ["Option A", "Option B", "Option C"] {
                        if ui.radio_value(&mut combo_value, option.to_string(), option).clicked() {
                            *self.state.combo_value.lock().unwrap() = combo_value.clone();
                            if let Some(signal) = &*self.state.event_signal.lock().unwrap() {
                                let _ = signal.send(Event::ComboSelected(combo_value.clone()));
                            }
                            self.state.log("ui", format!("Selected option: {}", combo_value));
                            self.state.save_config();
                        }
                    }
                });
            });

            ui.add_space(20.0);
            ui.collapsing("🎨 Log Colors", |ui| {
                let mut colors = self.state.colors.lock().unwrap().clone();
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("Clock Updates:");
                    changed |= ui.color_edit_button_srgba(&mut colors.clock).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Slider Events:");
                    changed |= ui.color_edit_button_srgba(&mut colors.slider).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Option A:");
                    changed |= ui.color_edit_button_srgba(&mut colors.option_a).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Option B:");
                    changed |= ui.color_edit_button_srgba(&mut colors.option_b).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Option C:");
                    changed |= ui.color_edit_button_srgba(&mut colors.option_c).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Time Format:");
                    changed |= ui.color_edit_button_srgba(&mut colors.time_format).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("Custom Events:");
                    changed |= ui.color_edit_button_srgba(&mut colors.custom_event).changed();
                });

                if changed {
                    *self.state.colors.lock().unwrap() = colors.clone();
                    self.state.save_config();

                    // Update all existing log entries with new colors
                    let mut logs = self.state.logs.lock().unwrap();
                    for log in logs.iter_mut() {
                        if log.source == "clock" {
                            log.color = Some(colors.clock);
                        } else if log.source == "ui" {
                            if log.message.contains("Slider value") {
                                log.color = Some(colors.slider);
                            } else if log.message.contains("Selected option: Option A") {
                                log.color = Some(colors.option_a);
                            } else if log.message.contains("Selected option: Option B") {
                                log.color = Some(colors.option_b);
                            } else if log.message.contains("Selected option: Option C") {
                                log.color = Some(colors.option_c);
                            }
                        }
                    }
                }
            });

            ui.add_space(20.0);
            if ui.button("Log Custom Event").clicked() {
                let colors = self.state.colors.lock().unwrap().clone();
                let custom_event = LogEntry {
                    timestamp: chrono::Local::now(),
                    source: "ui".to_string(),
                    message: "Custom Event".to_string(),
                    color: Some(colors.custom_event),
                };
                self.state.logs.lock().unwrap().push(custom_event);
            }
        });
    }
}



fn background_generator_thread(clock_signal: Signal<ClockMessage>, _ctx: egui::Context) {
    std::thread::spawn(move || {
        loop {
            let now = Local::now();
            if let Err(e) = clock_signal.send(ClockMessage::TimeUpdated(now.format("%H:%M:%S").to_string())) {
                eprintln!("Failed to send TimeUpdated message: {:?}", e);
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
                        eprintln!("Failed to parse config: {}", e);
                        Config {
                            slider_value: 0.5,
                            combo_value: "Option A".to_string(),
                            time_format: "24h".to_string(),
                            colors: LogColors::default(),
                        }
                    }
                }
            } else {
                eprintln!("Failed to read config file");
                Config {
                    slider_value: 0.5,
                    combo_value: "Option A".to_string(),
                    time_format: "24h".to_string(),
                    colors: LogColors::default(),
                }
            }
        } else {
            // Copy default config
            let default_config = Config {
                slider_value: 0.5,
                combo_value: "Option A".to_string(),
                time_format: "24h".to_string(),
                colors: LogColors::default(),
            };
            if let Ok(json_data) = serde_json::to_string_pretty(&default_config) {
                if let Err(e) = std::fs::write(&config_path, json_data) {
                    eprintln!("Failed to write default config: {}", e);
                }
            }
            default_config
        }
    };

    // Set up event handling system first
    let (event_signal, event_slot) = factory::create_signal_slot::<Event>(64);
    let (response_signal, response_slot) = factory::create_signal_slot::<Response>(64);
    
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
    let (clock_signal, clock_slot) = factory::create_signal_slot::<ClockMessage>(64);
    let now = Local::now().format("%H:%M:%S").to_string();
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
