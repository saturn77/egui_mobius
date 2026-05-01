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
        ui.heading(format!("Event Log ({total_events} events)"));

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

        let filtered_logs = self.state.filtered_logs.get();
        let colors = self.state.colors.get();

        let time_updates: Vec<_> = filtered_logs
            .iter()
            .filter(|entry| entry.source == "clock")
            .collect();
        let ui_events: Vec<_> = filtered_logs
            .iter()
            .filter(|entry| entry.source == "ui")
            .collect();

        ui.add_space(8.0);

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                egui::vec2(280.0, 0.0),
                egui::Layout::left_to_right(egui::Align::Center),
                |ui| {
                    ui.label(egui::RichText::new("Time Updates").strong().monospace());
                },
            );
            ui.add_space(20.0);
            ui.label(egui::RichText::new("UI Events").strong().monospace());
        });
        ui.add_space(8.0);

        let num_rows = time_updates.len().max(ui_events.len());
        let row_height = ui.text_style_height(&egui::TextStyle::Monospace);

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show_rows(ui, row_height, num_rows, |ui, row_range| {
                for row in row_range {
                    ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(280.0, row_height),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                if row < time_updates.len() {
                                    let entry = time_updates[time_updates.len() - 1 - row];
                                    let text = egui::RichText::new(format!(
                                        "[{}] {}",
                                        entry.timestamp.format("%H:%M:%S"),
                                        entry.message
                                    ))
                                    .monospace();
                                    ui.label(text.color(colors.clock));
                                }
                            },
                        );

                        ui.add_space(20.0);

                        ui.allocate_ui_with_layout(
                            egui::vec2(400.0, row_height),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                if row < ui_events.len() {
                                    let entry = ui_events[ui_events.len() - 1 - row];
                                    let text = egui::RichText::new(format!(
                                        "[{}] {}",
                                        entry.timestamp.format("%H:%M:%S"),
                                        entry.message
                                    ))
                                    .monospace();
                                    ui.label(
                                        text.color(entry.color.unwrap_or(colors.custom_event)),
                                    );
                                }
                            },
                        );
                    });
                }
            });
    }
}
