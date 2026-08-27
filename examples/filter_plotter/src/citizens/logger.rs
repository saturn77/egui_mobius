//! Lens-backed log panel. Reads from `SharedState::log` (a
//! `Dynamic<ReactiveEventLoggerState>`) and `SharedState::log_colors`;
//! the app writes via `citizens::append_log` and the
//! citizen-message drain.

use eframe::egui;
use egui_lens::ReactiveEventLogger;

use crate::state::SharedState;

use crate::tabs::LOGGER_ID;
use egui_citizen::citizen_panel;

citizen_panel!(LoggerPanel, LOGGER_ID);

impl LoggerPanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        // Force the panel to claim the full available rect — egui_dock
        // gives us the tab's full Ui, but lens's internal scroll area
        // would otherwise auto-shrink to its content size.
        let avail = ui.available_size_before_wrap();
        ui.allocate_ui_with_layout(
            avail,
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                let logger =
                    ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
                logger.show(ui);
            },
        );
    }
}
