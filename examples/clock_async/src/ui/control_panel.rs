use eframe::egui;
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
                let mut combo_value = self.state.combo_value.lock().unwrap().clone();
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

            // Custom Event Button
            ui.add_space(20.0);
            if ui.button("Log Custom Event").clicked() {
                let colors = self.state.colors.lock().unwrap().clone();
                let custom_event = crate::logger::LogEntry {
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
