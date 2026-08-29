//! Logger citizen — renders the app log: citizen lifecycle events from
//! the drain plus TX/connection events from the actions.

use eframe::egui;

use crate::state::SharedState;

use crate::tabs::LOGGER_ID;
use egui_citizen::citizen_panel;

citizen_panel!(LoggerPanel, LOGGER_ID);

impl LoggerPanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let lines = state.log.get();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &lines {
                    ui.label(egui::RichText::new(line).monospace().size(12.0));
                }
            });
    }
}
