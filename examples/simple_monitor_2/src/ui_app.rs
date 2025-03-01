use eframe;
use egui;
use egui_mobius::types::{Enqueue, Value}; 
use crate::UiCommand;


pub struct App {
    pub logger_text     : Value<String>,
    pub command_sender  : Enqueue<UiCommand>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            ui.horizontal(|ui| {
                let cascade_commands = vec![UiCommand::FirstTask, UiCommand::SecondTask];
                let cascade_first_second = {
                    let commands = cascade_commands.clone();
                    UiCommand::CascadeFirstSecond(commands)
                };
                UiCommand::generate_buttons(ui, &self.command_sender, vec![
                    ("First Task", UiCommand::FirstTask),
                    ("Second Task", UiCommand::SecondTask),
                    ("Clear Terminal", UiCommand::ClearTerminal),
                    ("About", UiCommand::About), 
                    ("Cascade First Second", cascade_first_second.clone()),
                ]);
            });

            let mut _scroller = egui::ScrollArea::vertical()
                .id_salt("terminal_scroller")
                .stick_to_bottom(false)
                .max_height(400.0_f32)
                .show(ui, |ui| {
                    egui::TextEdit::multiline(&mut *self.logger_text.lock().unwrap())
                        .id(egui::Id::new("terminal"))
                        .text_color(egui::Color32::YELLOW)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .interactive(true)
                        .desired_rows(20)
                        .lock_focus(true)
                        .desired_width(600.)
                        .show(ui);
                });
        });
    }
}
