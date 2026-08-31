//! Monitor citizen — port picker, baud, connect/disconnect. Every
//! intent goes onto the outbox; main.rs drains it and forwards through
//! `monitor_actions::handle` to the serial backend. Sending lives in
//! the Console, terminal-style.

use eframe::egui;

use crate::backend::LineEnding;
use crate::messages::AppMessage;
use crate::state::SharedState;

use crate::tabs::MONITOR_ID;
use egui_citizen::citizen_panel;

// `selected_port` / `baud` — the panel's atoms; `outbox` — outgoing
// intents drained by main.rs.
citizen_panel!(MonitorPanel, MONITOR_ID,
    selected_port: String = String::new(),
    baud: u32 = 115_200,
    outbox: Vec<AppMessage> = Vec::new(),
);

const BAUD_RATES: [u32; 6] = [9_600, 19_200, 38_400, 57_600, 115_200, 921_600];

impl MonitorPanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        ui.heading("Serial Monitor");
        ui.add_space(8.0);

        let connected = state.connected.get();
        let ports = state.ports.get();

        // ── Port selection ────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!connected, |ui| {
                egui::ComboBox::from_label("port")
                    .selected_text(if self.selected_port.is_empty() {
                        "— select —"
                    } else {
                        &self.selected_port
                    })
                    .show_ui(ui, |ui| {
                        for p in &ports {
                            ui.selectable_value(&mut self.selected_port, p.clone(), p);
                        }
                    });
                if ui.button("Refresh").clicked() {
                    self.outbox.push(AppMessage::RefreshPorts);
                }
            });
        });
        if ports.is_empty() {
            ui.label(
                egui::RichText::new(
                    "no ports found — plug in a device, or make a virtual pair:\n\
                     socat -d -d pty,raw,echo=0 pty,raw,echo=0",
                )
                .weak()
                .monospace(),
            );
        }

        // ── Baud + connect ────────────────────────────────────────────
        ui.horizontal(|ui| {
            ui.add_enabled_ui(!connected, |ui| {
                egui::ComboBox::from_label("baud")
                    .selected_text(self.baud.to_string())
                    .show_ui(ui, |ui| {
                        for b in BAUD_RATES {
                            ui.selectable_value(&mut self.baud, b, b.to_string());
                        }
                    });
            });

            if connected {
                if ui.button("Disconnect").clicked() {
                    self.outbox.push(AppMessage::Disconnect);
                }
            } else {
                let can_connect = !self.selected_port.is_empty();
                if ui
                    .add_enabled(can_connect, egui::Button::new("Connect"))
                    .clicked()
                {
                    self.outbox.push(AppMessage::Connect {
                        port: self.selected_port.clone(),
                        baud: self.baud,
                    });
                }
            }
        });

        // ── TX line ending ────────────────────────────────────────────
        ui.horizontal(|ui| {
            let mut ending = state.line_ending.get();
            let before = ending;
            egui::ComboBox::from_label("line end")
                .selected_text(ending.label())
                .show_ui(ui, |ui| {
                    for e in LineEnding::ALL {
                        ui.selectable_value(&mut ending, e, e.label());
                    }
                });
            if ending != before {
                state.line_ending.set(ending);
            }
        });

        ui.add_space(4.0);
        ui.label(egui::RichText::new(state.status.get()).monospace().weak());

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("type in the Console tab to transmit — Enter sends, ↑/↓ recall")
                .weak(),
        );

        ui.add_space(4.0);
        if ui.button("Clear console").clicked() {
            self.outbox.push(AppMessage::ClearRx);
        }
    }
}
