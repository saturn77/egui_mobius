
use crate::state::AppState;
use eframe::egui;
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
        // Get current state
        let total_events = self.state.log_count.get();
        ui.heading(format!("Event Log ({} events)", total_events));

        let log_filters = self.state.log_filters.get();
        ui.horizontal(|ui| {
            ui.label("Source filter:");
            for source in [&"ui", &"clock"] {
                let source = *source;
                let selected = log_filters.contains(&source.to_string());
                if ui.selectable_label(selected, source).clicked() {
                    let mut filters = self.state.log_filters.get();
                    if selected {
                        filters.retain(|s| s != source);
                    } else {
                        filters.push(source.to_string());
                    }
                    self.state.log_filters.set(filters);
                }
            }
        });

        if ui.button("Clear Logger").clicked() {
            let mut logs = self.state.logs.get();
            logs.clear();
            self.state.logs.set(logs);
        }

        // Get logs and colors
        let filtered_logs = self.state.filtered_logs.get();
        let colors = self.state.colors.get();

        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.add_space(4.0);

                // Headers
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Time Updates").strong().monospace());
                    ui.add_space(20.0);
                    ui.label(egui::RichText::new("UI Events").strong().monospace());
                });
                ui.add_space(8.0);

                // Sort logs by source
                let time_updates: Vec<_> = filtered_logs.iter()
                    .filter(|entry| entry.source == "clock")
                    .collect();
                let ui_events: Vec<_> = filtered_logs.iter()
                    .filter(|entry| entry.source == "ui")
                    .collect();

                // Display entries side by side
                ui.horizontal(|ui| {
                    // Time Updates column
                    ui.vertical(|ui| {
                        ui.set_min_width(280.0);
                        for entry in time_updates.iter().rev() {
                            let text = egui::RichText::new(
                                format!("[{}] {}", entry.timestamp.format("%H:%M:%S"), entry.message)
                            ).monospace();
                            ui.label(text.color(colors.clock));
                        }
                    });

                    // Spacer
                    ui.add_space(20.0);

                    // UI Events column
                    ui.vertical(|ui| {
                        ui.set_min_width(400.0);
                        for entry in ui_events.iter().rev() {
                            let text = egui::RichText::new(
                                format!("[{}] {}", entry.timestamp.format("%H:%M:%S"), entry.message)
                            ).monospace();
                            ui.label(text.color(entry.color.unwrap_or(colors.custom_event)));
                        }
                    });
                });
            });
    }
}
