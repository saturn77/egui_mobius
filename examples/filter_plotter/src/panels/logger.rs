//! Lens-backed log panel. Reads from `SharedState::log` (a
//! `Dynamic<ReactiveEventLoggerState>`); the dispatcher writes via
//! `dispatcher::append_log` and the citizen-message drain.

use eframe::egui;
use egui_lens::ReactiveEventLogger;

use crate::state::SharedState;

pub struct LoggerPanel {}

impl LoggerPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let logger = ReactiveEventLogger::new(&state.log);
        logger.show(ui);
    }
}
