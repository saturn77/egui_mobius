//! Scrolling log panel. Reads from `SharedState::log`; populated by the
//! drain loop in main.rs.

use eframe::egui;

use crate::state::SharedState;

pub struct LoggerPanel {}

impl LoggerPanel {
    pub fn new() -> Self {
        Self {}
    }

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
