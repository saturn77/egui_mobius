use crate::state::AppState;
use crate::types::{LogColors, LogEntry};
use eframe::egui;
use egui_mobius_widgets::{StyledButton, StatefulButton};

pub struct ControlPanel<'a> {
    state: &'a AppState,
}

impl<'a> ControlPanel<'a> {
    fn update_log_colors(&self, colors: &LogColors) {
        let mut logs = self.state.logs.get();
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
        self.state.logs.set(logs);
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
            // Clock Display with formatted time
            ui.add_space(10.0);
            let time_str = self.state.formatted_time.get();
            ui.heading(time_str);
            
            ui.add_space(20.0);
            ui.heading("Clock Settings");
            ui.add_space(10.0);

            // Time Format Section
            ui.collapsing("⚙️ Time Format", |ui| {
                ui.horizontal(|ui| {
                    let mut use_24h = self.state.use_24h.get();
                    if ui.radio_value(&mut use_24h, false, "12-hour").clicked() {
                        self.state.use_24h.set(use_24h);
                        self.state.log("ui", "Changed time format to 12-hour".to_string());
                        self.state.save_config();
                    }
                    if ui.radio_value(&mut use_24h, true, "24-hour").clicked() {
                        self.state.use_24h.set(use_24h);
                        self.state.log("ui", "Changed time format to 24-hour".to_string());
                        self.state.save_config();
                    }
                });
            });

            // Controls Section
            ui.add_space(20.0);
            ui.collapsing("🎛️ Controls", |ui| {
                let mut slider_value = self.state.slider_value.get();
                if ui.add(egui::Slider::new(&mut slider_value, 0.0..=100.0).text("Value")).changed() {
                    self.state.slider_value.set(slider_value);
                    self.state.log("ui", format!("Slider value changed to {}", slider_value));
                    self.state.save_config();
                }

                ui.add_space(10.0);

                // Combo box
                let combo_value = self.state.combo_value.get();
                egui::ComboBox::from_label("Select an option")
                    .selected_text(combo_value.clone())
                    .show_ui(ui, |ui| {
                        for option in ["Option A", "Option B", "Option C"].iter() {
                            if ui.selectable_label(combo_value == *option, *option).clicked() {
                                self.state.combo_value.set(option.to_string());
                                self.state.log("ui", format!("Selected option: {}", option));
                                self.state.save_config();
                            }
                        }
                    });

                // Button Colors Section
                ui.add_space(20.0);
                ui.collapsing("🎨 Button Colors", |ui| {
                    let mut button_colors = self.state.button_colors.get();
                    let mut changed = false;

                    ui.horizontal(|ui| {
                        ui.label("Run State:");
                        changed |= ui.color_edit_button_srgba(&mut button_colors.run_state).changed();
                    });

                    ui.horizontal(|ui| {
                        ui.label("Stop State:");
                        changed |= ui.color_edit_button_srgba(&mut button_colors.stop_state).changed();
                    });

                    if changed {
                        self.state.button_colors.set(button_colors);
                        self.state.save_config();
                    }
                });

                // Log Colors Section
                ui.add_space(20.0);
                ui.collapsing("🎨 Log Colors", |ui| {
                    let mut colors = self.state.colors.get();
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
                        ui.label("Custom Event:");
                        changed |= ui.color_edit_button_srgba(&mut colors.custom_event).changed();
                    });

                    ui.horizontal(|ui| {
                        ui.label("Run/Stop:");
                        changed |= ui.color_edit_button_srgba(&mut colors.run_stop_log).changed();
                    });

                    if changed {
                        self.state.colors.set(colors.clone());
                        self.update_log_colors(&colors);
                    }
                });

                ui.add_space(20.0);

                // Custom Event Button
                let custom_button = StyledButton::new("Custom Event")
                    .margin(egui::Vec2::new(4.0, 2.0))
                    .min_size(egui::vec2(120.0, 24.0));
                
                if custom_button.show(ui).clicked() {
                    let colors = self.state.colors.get();
                    let custom_event = LogEntry {
                        timestamp: chrono::Local::now(),
                        source: "ui".to_string(),
                        message: "Custom Event triggered".to_string(),
                        color: Some(colors.custom_event),
                    };
                    let mut logs = self.state.logs.get();
                    logs.push_back(custom_event);
                    self.state.logs.set(logs);
                }

                ui.add_space(5.0);

                // Styled Button
                let styled_button = StyledButton::new("Styled Button")
                    .margin(egui::Vec2::new(4.0, 2.0))
                    .min_size(egui::vec2(120.0, 24.0));
                
                if styled_button.show(ui).clicked() {
                    let colors = self.state.colors.get();
                    let custom_event = LogEntry {
                        timestamp: chrono::Local::now(),
                        source: "ui".to_string(),
                        message: "Styled Button clicked".to_string(),
                        color: Some(colors.custom_event),
                    };
                    let mut logs = self.state.logs.get();
                    logs.push_back(custom_event);
                    self.state.logs.set(logs);
                }

                ui.add_space(10.0);

                // Buffer size control
                ui.group(|ui| {
                    ui.label("Log Buffer Size:");
                    let mut buffer_size = self.state.buffer_size.get();
                    if ui.add(egui::DragValue::new(&mut buffer_size)
                        .range(100..=10000)
                        .speed(100)
                    ).changed() {
                        self.state.buffer_size.set(buffer_size);
                        
                        // Log the buffer size change
                        let colors = self.state.colors.get();
                        let custom_event = LogEntry {
                            timestamp: chrono::Local::now(),
                            source: "ui".to_string(),
                            message: format!("Buffer size changed to {}", buffer_size),
                            color: Some(colors.custom_event),
                        };
                        let mut logs = self.state.logs.get();
                        logs.push_back(custom_event);
                        self.state.logs.set(logs);
                    }
                });

                ui.add_space(10.0);

                // RUN/STOP Button
                let mut button_started = self.state.button_started.get();
                let button_colors = self.state.button_colors.get();
                
                let mut stateful_button = StatefulButton::new()
                    .margin(egui::Vec2::new(4.0, 2.0))
                    .min_size(egui::vec2(120.0, 24.0))
                    .run_color(button_colors.run_state)
                    .stop_color(button_colors.stop_state);
                
                stateful_button.set_started(button_started);
                
                if stateful_button.show(ui).clicked() {
                    button_started = !button_started;
                    self.state.button_started.set(button_started);
                    let message = if button_started {
                        "Process Started"
                    } else {
                        "Process Stopped"
                    };

                    let colors = self.state.colors.get();
                    let custom_event = LogEntry {
                        timestamp: chrono::Local::now(),
                        source: "ui".to_string(),
                        message: message.to_string(),
                        color: Some(colors.run_stop_log),
                    };
                    let mut logs = self.state.logs.get();
                    logs.push_back(custom_event);
                    self.state.logs.set(logs);
                }
            });
        });
    }
}
