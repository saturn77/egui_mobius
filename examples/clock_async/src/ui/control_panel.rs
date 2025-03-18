use eframe::egui;
use egui_mobius_widgets::{StyledButton, StatefulButton};
use crate::state::AppState;
use crate::types::Event;

pub struct ControlPanel<'a> {
    state: &'a AppState,
}

impl<'a> ControlPanel<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub fn render(ui: &mut egui::Ui, state: &'a AppState) {
        let mut panel = Self::new(state);
        panel.ui(ui);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Clock Display
            ui.add_space(10.0);
            let time_str = self.state.current_time.lock().unwrap().clone();
            ui.heading(time_str);
            
            ui.add_space(20.0);
            ui.heading("Clock Settings");
            ui.add_space(10.0);

            // Time Format Section
            ui.collapsing("⚙️ Time Format", |ui| {
                ui.horizontal(|ui| {
                    let mut use_24h = *self.state.use_24h.lock().unwrap();
                    if ui.radio_value(&mut use_24h, false, "12-hour").clicked() {
                        *self.state.use_24h.lock().unwrap() = use_24h;
                        self.state.log("ui", "Changed time format to 12-hour".to_string());
                        self.state.save_config();
                    }
                    if ui.radio_value(&mut use_24h, true, "24-hour").clicked() {
                        *self.state.use_24h.lock().unwrap() = use_24h;
                        self.state.log("ui", "Changed time format to 24-hour".to_string());
                        self.state.save_config();
                    }
                });
            });

            // Controls Section
            ui.add_space(20.0);
            ui.collapsing("🎛️ Controls", |ui| {
                let mut slider_value = *self.state.slider_value.lock().unwrap();
                if ui.add(egui::Slider::new(&mut slider_value, 0.0..=100.0).text("Value")).changed() {
                    *self.state.slider_value.lock().unwrap() = slider_value;
                    if let Some(signal) = &*self.state.event_signal.lock().unwrap() {
                        let _ = signal.send(Event::SliderChanged(slider_value));
                    }
                    self.state.log("ui", format!("Slider value changed to {}", slider_value));
                    self.state.save_config();
                }

                ui.add_space(10.0);

                // Combo box
                let combo_value = self.state.combo_value.lock().unwrap().clone();
                egui::ComboBox::from_label("Select an option")
                    .selected_text(combo_value.clone())
                    .show_ui(ui, |ui| {
                        for option in ["Option A", "Option B", "Option C"].iter() {
                            if ui.selectable_label(combo_value == *option, *option).clicked() {
                                *self.state.combo_value.lock().unwrap() = option.to_string();
                                if let Some(signal) = &*self.state.event_signal.lock().unwrap() {
                                    let _ = signal.send(Event::ComboSelected(option.to_string()));
                                }
                                self.state.log("ui", format!("Selected option: {}", option));
                            }
                        }
                    });
            });

            // Button Colors Section
            ui.add_space(20.0);
            ui.collapsing("🎨 Button Colors", |ui| {
                let mut button_colors = self.state.button_colors.lock().unwrap().clone();
                let mut changed = false;

                ui.horizontal(|ui| {
                    ui.label("RUN State:");
                    changed |= ui.color_edit_button_srgba(&mut button_colors.run_state).changed();
                });

                ui.horizontal(|ui| {
                    ui.label("STOP State:");
                    changed |= ui.color_edit_button_srgba(&mut button_colors.stop_state).changed();
                });

                if changed {
                    *self.state.button_colors.lock().unwrap() = button_colors;
                    self.state.save_config();
                }
            });

            // Log Colors Section
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

                ui.horizontal(|ui| {
                    ui.label("RUN/STOP Log:");
                    changed |= ui.color_edit_button_srgba(&mut colors.run_stop_log).changed();
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

            // Button container with vertical layout
            ui.vertical(|ui| {
                ui.add_space(20.0);

                // Custom Event Button (now styled)
                let custom_button = StyledButton::new("Custom Event")
                    .hover_color(egui::Color32::from_rgb(100, 200, 255))
                    .normal_color(egui::Color32::from_gray(128))
                    .rounding(8.0)
                    .margin(egui::Vec2::new(10.0, 5.0))
                    .min_size(egui::vec2(120.0, 27.0));
                
                if custom_button.show(ui).clicked() {
                    let custom_event = crate::logger::LogEntry {
                        timestamp: chrono::Local::now(),
                        source: "ui".to_string(),
                        message: "Custom Event".to_string(),
                        color: Some(egui::Color32::from_rgb(100, 200, 255)),
                    };
                    self.state.logs.lock().unwrap().push(custom_event);
                }

                ui.add_space(10.0);

                // Green Styled Button
                let styled_button = StyledButton::new("Green Styled Event")
                    .hover_color(egui::Color32::from_rgb(0, 255, 0)) // Bright green hover
                    .normal_color(egui::Color32::from_gray(128))
                    .rounding(8.0)
                    .margin(egui::Vec2::new(10.0, 5.0))
                    .min_size(egui::vec2(120.0, 27.0));
                
                if styled_button.show(ui).clicked() {
                    let custom_event = crate::logger::LogEntry {
                        timestamp: chrono::Local::now(),
                        source: "ui".to_string(),
                        message: "Green Custom Event".to_string(),
                        color: Some(egui::Color32::from_rgb(0, 255, 0)),
                    };
                    self.state.logs.lock().unwrap().push(custom_event);
                }

                ui.add_space(10.0);

                // Stateful Start/Stop Button
                // Get button colors and release lock immediately
                let (run_color, stop_color) = {
                    let colors = self.state.button_colors.lock().unwrap();
                    (colors.run_state, colors.stop_state)
                };
                
                // Get log color
                let log_color = self.state.colors.lock().unwrap().run_stop_log;
                
                let mut stateful_button = StatefulButton::new()
                    .margin(egui::Vec2::new(8.5, 4.25))
                    .rounding(8.0)
                    .min_size(egui::vec2(120.0, 27.0))
                    .run_color(run_color)
                    .stop_color(stop_color);
                
                // Set the button's state from our stored state
                let mut button_started = self.state.button_started.lock().unwrap();
                stateful_button.set_started(*button_started);
                
                if stateful_button.show(ui).clicked() {
                    // Toggle the state and get the new state
                    *button_started = !*button_started;
                    let is_started = *button_started;
                    drop(button_started); // Release the lock early
                    
                    // Get the appropriate color
                    // Use the same log color for both states
                    let color = log_color;
                    
                    let message = if is_started {
                        "Process Started"
                    } else {
                        "Process Stopped"
                    };
                    
                    let custom_event = crate::logger::LogEntry {
                        timestamp: chrono::Local::now(),
                        source: "ui".to_string(),
                        message: message.to_string(),
                        color: Some(color),
                    };
                    self.state.logs.lock().unwrap().push(custom_event);
                }
            });
        });
    }
}
