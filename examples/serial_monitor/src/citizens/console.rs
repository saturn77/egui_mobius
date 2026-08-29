//! Console citizen — renders the received lines. Pure reader: the
//! serial worker is the one writer of `rx_lines`, and Path A carries
//! each new line here on the next frame.

use eframe::egui;

use crate::state::SharedState;

use crate::tabs::CONSOLE_ID;
use egui_citizen::citizen_panel;

citizen_panel!(ConsolePanel, CONSOLE_ID);

impl ConsolePanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let lines = state.rx_lines.get();
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                for line in &lines {
                    ui.label(egui::RichText::new(line).monospace());
                }
                if lines.is_empty() {
                    ui.label(
                        egui::RichText::new("(nothing received yet)").weak(),
                    );
                }
            });
    }
}
