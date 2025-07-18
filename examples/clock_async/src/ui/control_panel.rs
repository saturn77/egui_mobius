use eframe::egui;
use egui_mobius_widgets::{StyledButton, StatefulButton};

use crate::state::AppState;
use crate::types::Event;

pub struct ControlPanel<'a> {
    state: &'a AppState,
}

impl<'a> ControlPanel<'a> {
    fn update_log_colors(&self, colors: &crate::logger::LogColors) {
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
                } else if log.message.contains("Process") {
                    log.color = Some(colors.run_stop_log);
                } else if log.message.contains("Custom Event") {
                    log.color = Some(colors.custom_event);
                }
            }
        }
        self.state.save_config();
    }
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
                    self.state.log("ui", format!("Slider value changed to {slider_value}"));
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
                                self.state.log("ui", format!("Selected option: {option}"));
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
                let mut colors = self.state.colors.lock().unwrap();
                let mut changed = false;
                let mut colors_clone = colors.clone();

                ui.horizontal(|ui| {
                    ui.label("Clock Updates:");
                    changed |= ui.color_edit_button_srgba(&mut colors.clock).changed();
                    colors_clone.clock = colors.clock;
                });

                ui.horizontal(|ui| {
                    ui.label("Slider Events:");
                    changed |= ui.color_edit_button_srgba(&mut colors.slider).changed();
                    colors_clone.slider = colors.slider;
                });

                ui.horizontal(|ui| {
                    ui.label("Option A:");
                    changed |= ui.color_edit_button_srgba(&mut colors.option_a).changed();
                    colors_clone.option_a = colors.option_a;
                });

                ui.horizontal(|ui| {
                    ui.label("Option B:");
                    changed |= ui.color_edit_button_srgba(&mut colors.option_b).changed();
                    colors_clone.option_b = colors.option_b;
                });

                ui.horizontal(|ui| {
                    ui.label("Option C:");
                    changed |= ui.color_edit_button_srgba(&mut colors.option_c).changed();
                    colors_clone.option_c = colors.option_c;
                });

                ui.horizontal(|ui| {
                    ui.label("Time Format:");
                    changed |= ui.color_edit_button_srgba(&mut colors.time_format).changed();
                    colors_clone.time_format = colors.time_format;
                });

                ui.horizontal(|ui| {
                    ui.label("Custom Events:");
                    changed |= ui.color_edit_button_srgba(&mut colors.custom_event).changed();
                    colors_clone.custom_event = colors.custom_event;
                });

                ui.horizontal(|ui| {
                    ui.label("RUN/STOP Events:");
                    changed |= ui.color_edit_button_srgba(&mut colors.run_stop_log).changed();
                    colors_clone.run_stop_log = colors.run_stop_log;
                });

                drop(colors);
                if changed {
                    self.update_log_colors(&colors_clone);
                }
            });

            // Button container with vertical layout
            ui.vertical(|ui| {
                ui.add_space(10.0);

                // Custom Event Button
                let custom_button = StyledButton::new("Custom Event")
                    .hover_color(egui::Color32::from_rgb(100, 200, 255))
                    .normal_color(egui::Color32::from_gray(128))
                    .rounding(4.0)
                    .margin(egui::Vec2::new(4.0, 2.0))
                    .min_size(egui::vec2(120.0, 24.0));
                
                if custom_button.show(ui).clicked() {
                    let colors = self.state.colors.lock().unwrap();
                    let custom_event = crate::logger::LogEntry {
                        timestamp: chrono::Local::now(),
                        source: "ui".to_string(),
                        message: "Custom Event".to_string(),
                        color: Some(colors.custom_event),
                    };
                    self.state.logs.lock().unwrap().push_back(custom_event);
                }

                ui.add_space(5.0);

                // Custom Event 2 Button
                let styled_button = StyledButton::new("Custom Event 2")
                    .hover_color(egui::Color32::from_rgb(100, 200, 255))
                    .normal_color(egui::Color32::from_gray(128))
                    .rounding(4.0)
                    .margin(egui::Vec2::new(4.0, 2.0))
                    .min_size(egui::vec2(120.0, 24.0));
                
                if styled_button.show(ui).clicked() {
                    let colors = self.state.colors.lock().unwrap();
                    let custom_event = crate::logger::LogEntry {
                        timestamp: chrono::Local::now(),
                        source: "ui".to_string(),
                        message: "Custom Event 2".to_string(),
                        color: Some(colors.custom_event),
                    };
                    self.state.logs.lock().unwrap().push_back(custom_event);
                }

                ui.add_space(10.0);

                // Buffer size control
                ui.group(|ui| {
                    ui.label("Log Buffer Size:");
                    let mut buffer_size = *self.state.buffer_size.lock().unwrap();
                    if ui.add(egui::DragValue::new(&mut buffer_size)
                        .range(100..=10000)
                        .speed(100)
                        .prefix("Max entries: ")
                    ).changed() {
                        *self.state.buffer_size.lock().unwrap() = buffer_size;
                        
                        // Log the buffer size change
                        let colors = self.state.colors.lock().unwrap();
                        let custom_event = crate::logger::LogEntry {
                            timestamp: chrono::Local::now(),
                            source: "ui".to_string(),
                            message: format!("Log buffer size changed to {buffer_size}"),
                            color: Some(colors.custom_event),
                        };
                        self.state.logs.lock().unwrap().push_back(custom_event);
                    }
                });

                ui.add_space(10.0);

                // RUN/STOP Button
                let mut button_started = self.state.button_started.lock().unwrap();
                let button_colors = self.state.button_colors.lock().unwrap();
                
                let mut stateful_button = StatefulButton::new()
                    .margin(egui::Vec2::new(4.0, 2.0))
                    .rounding(4.0)
                    .min_size(egui::vec2(120.0, 24.0))
                    .run_color(button_colors.run_state)
                    .stop_color(button_colors.stop_state);
                
                stateful_button.set_started(*button_started);
                
                if stateful_button.show(ui).clicked() {
                    *button_started = !*button_started;
                    let message = if *button_started {
                        "Process Started"
                    } else {
                        "Process Stopped"
                    };

                    let colors = self.state.colors.lock().unwrap();
                    let custom_event = crate::logger::LogEntry {
                        timestamp: chrono::Local::now(),
                        source: "ui".to_string(),
                        message: message.to_string(),
                        color: Some(colors.run_stop_log),
                    };
                    self.state.logs.lock().unwrap().push_back(custom_event);
                }
            });
        });
    }
}
