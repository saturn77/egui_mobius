//! Console citizen — a shell over the serial port, patterned on
//! blaze2-gui's Shell. The scrollback and the input line live together:
//! a frameless `> ` prompt trails the last received line and sticks to
//! the bottom. Enter sends (via the outbox → `console_actions`);
//! Up/Down recall history. TX lines are echoed into the scrollback by
//! the serial worker, so the stream reads like a real terminal session.

use eframe::egui;

use crate::messages::AppMessage;
use crate::state::SharedState;
use crate::theme::TokyoNight;

use crate::tabs::CONSOLE_ID;
use egui_citizen::citizen_panel;

citizen_panel!(ConsolePanel, CONSOLE_ID,
    input: String = String::new(),
    history: Vec<String> = Vec::new(),
    hist_pos: Option<usize> = None,
    outbox: Vec<AppMessage> = Vec::new(),
);

impl ConsolePanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let connected = state.connected.get();
        let lines = state.rx_lines.get();
        let bg = ui.visuals().extreme_bg_color;
        let input_id = ui.make_persistent_id("serial_monitor_console_input");

        egui::Frame::new().fill(bg).inner_margin(8.0).show(ui, |ui| {
            ui.style_mut().visuals.extreme_bg_color = bg;
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &lines {
                        if line.is_empty() {
                            // A blank line from a bare LF/CR — hold the row.
                            ui.label(egui::RichText::new(" ").monospace());
                            continue;
                        }
                        let rich = if line.starts_with("> ") {
                            // TX echo, written by the worker at transmit time.
                            egui::RichText::new(line).color(TokyoNight::GREEN)
                        } else if line.starts_with('[') {
                            // Worker-reported errors ("[tx error] ...").
                            egui::RichText::new(line).color(TokyoNight::RED)
                        } else {
                            egui::RichText::new(line).color(TokyoNight::FG)
                        };
                        ui.label(rich.monospace());
                    }

                    // The prompt trails the scrollback, terminal-style.
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 0.0;
                        let prompt = if connected { "> " } else { "· " };
                        ui.label(
                            egui::RichText::new(prompt)
                                .color(TokyoNight::GREEN)
                                .monospace()
                                .strong(),
                        );
                        let resp = ui.add_enabled(
                            connected,
                            egui::TextEdit::singleline(&mut self.input)
                                .id(input_id)
                                .desired_width(ui.available_width())
                                .font(egui::TextStyle::Monospace)
                                .frame(egui::Frame::NONE)
                                .hint_text(if connected {
                                    ""
                                } else {
                                    "not connected — see the Monitor panel"
                                }),
                        );

                        if resp.has_focus() {
                            ui.input(|i| {
                                if i.key_pressed(egui::Key::ArrowUp) {
                                    self.recall_history(-1);
                                } else if i.key_pressed(egui::Key::ArrowDown) {
                                    self.recall_history(1);
                                }
                            });
                        }

                        let entered = resp.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter));
                        if entered {
                            let line = self.input.trim().to_string();
                            if !line.is_empty() {
                                self.input.clear();
                                self.history.push(line.clone());
                                self.hist_pos = None;
                                self.outbox.push(AppMessage::Send(line));
                            }
                            ui.memory_mut(|m| m.request_focus(input_id));
                        }

                        // Keep the cursor in the console so you can just
                        // type, unless another widget wants the keyboard.
                        if connected
                            && !resp.has_focus()
                            && !ui.ctx().egui_wants_keyboard_input()
                            && !ui.ctx().any_popup_open()
                        {
                            ui.memory_mut(|m| m.request_focus(input_id));
                        }
                    });
                });
        });
    }

    fn recall_history(&mut self, dir: i32) {
        let n = self.history.len();
        if n == 0 {
            return;
        }
        let pos = match (self.hist_pos, dir) {
            (None, -1) => n - 1,
            (Some(p), -1) => p.saturating_sub(1),
            (Some(p), 1) if p + 1 < n => p + 1,
            (Some(_), 1) => {
                self.hist_pos = None;
                self.input.clear();
                return;
            }
            _ => return,
        };
        self.hist_pos = Some(pos);
        self.input = self.history[pos].clone();
    }
}
