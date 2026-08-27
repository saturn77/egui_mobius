//! Scrolling log panel — reads `SharedState::log`. Populated by the
//! drain loop in main.rs (citizen lifecycle + AppMessage trace) and by
//! the result-slot handler (backend completions).

use eframe::egui;

use crate::state::SharedState;

use crate::tabs::LOGGER_ID;
use egui_citizen::citizen_panel;

citizen_panel!(LoggerPanel, LOGGER_ID);

impl LoggerPanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        ui.heading("Log");
        ui.add_space(4.0);

        let log = state.log.get();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if log.is_empty() {
                    ui.weak("(no events yet)");
                } else {
                    for line in log.iter() {
                        ui.monospace(line);
                    }
                }
            });
    }
}
