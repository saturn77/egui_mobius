use eframe::egui;
use crate::state::AppState;
use crate::logger::LogEntry;

pub struct LoggerPanel<'a> {
    state: &'a AppState,
}

impl<'a> LoggerPanel<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }

    pub fn render(ui: &mut egui::Ui, state: &'a AppState) {
        let mut panel = Self::new(state);
        panel.ui(ui);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let total_events = self.state.logs.lock().unwrap().len();
        ui.heading(format!("Event Log ({total_events} events)"));

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

        ui.add_space(8.0);

        // Create a scrollable area for the log entries
        egui::ScrollArea::vertical()
            .id_salt("log_scroll")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                // Headers
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Time Updates").strong().monospace());
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("UI Events").strong().monospace());
                });
                ui.add_space(8.0);

            // Split entries into two columns
            let mut time_updates: Vec<&LogEntry> = Vec::new();
            let mut ui_events: Vec<&LogEntry> = Vec::new();

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
            ui.horizontal(|ui| {
                // Time Updates column
                ui.vertical(|ui| {
                    ui.set_min_width(280.0);
                    for entry in time_updates.iter() {
                        let text = egui::RichText::new(entry.formatted()).monospace();
                        ui.label(text.color(colors.clock));
                    }
                });

                // Spacer
                ui.add_space(20.0);

                // UI Events column
                ui.vertical(|ui| {
                    ui.set_min_width(400.0);
                    for entry in ui_events.iter() {
                        // Use the entry's custom color if available, otherwise determine from message
                        let color = entry.color.unwrap_or_else(|| {
                            if entry.message.contains("Slider value") {
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
                            }
                        });
                        let text = egui::RichText::new(entry.formatted()).monospace();
                        ui.label(text.color(color));
                    }
                });
            });
        });
    }
}
