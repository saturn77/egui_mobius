//! Result panel — displays the latest backend value, with a spinner
//! while a request is in flight. Reads `last_result` and `in_flight`
//! through their shared `Dynamic<T>` handles; both are written by the
//! result-slot handler off the UI thread.

use eframe::egui;

use crate::state::SharedState;

use crate::tabs::RESULT_ID;
use egui_citizen::citizen_panel;

citizen_panel!(ResultPanel, RESULT_ID);

impl ResultPanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        ui.heading("Result");
        ui.add_space(8.0);

        if state.in_flight.get() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("waiting for backend…");
            });
            ui.add_space(8.0);
        }

        ui.label("Latest value:");
        ui.add_space(4.0);
        ui.heading(format!("{:.6}", state.last_result.get()));

        ui.add_space(12.0);
        ui.weak("Updated by the result-slot handler off the UI thread.");
    }
}
