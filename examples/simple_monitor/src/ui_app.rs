use eframe;
use egui;
use mobius_egui::types::{MobiusString, MobiusEnque}; 
use mobius_egui::Signal;
use crate::Command;

pub struct App {
    pub logger_text     : MobiusString,
    pub command_sender  : MobiusEnque<Command>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            // buttons to send commands
            ui.horizontal(|ui| {
                if ui.button("First Task").clicked() {
                    println!("First Task button clicked.");
                    Signal!(self.command_sender, Command::FirstTask);
                }
                if ui.button("Second Task").clicked() {
                    println!("Second Task button clicked.");
                    Signal!(self.command_sender, Command::SecondTask);
                }
                if ui.button("Clear Terminal").clicked() {
                    println!("Clear Terminal button clicked.");
                    Signal!(self.command_sender, Command::ClearTerminal);
                }
                if ui.button("Send Multiple Commands").clicked() {
                    println!("Send Multiple Commands button clicked.");
                    let commands = vec![Command::FirstTask, Command::SecondTask];
                    Signal!(self.command_sender, commands, multiple);
                }
                if ui.button("About").clicked() {
                    println!("About button clicked.");
                    Signal!(self.command_sender, Command::About);
                }
            });

            //*******************************************************************
            // Main Scroller for Terminal Window
            //*******************************************************************

            let scroller_text_color: egui::Color32 = egui::Color32::GREEN;

            let mut _scroller = egui::ScrollArea::vertical()
                .id_salt("terminal_scroller")
                .stick_to_bottom(false)
                .max_height(400.0_f32)
                .show(ui, |ui| {
                    egui::TextEdit::multiline(&mut *self.logger_text.lock().unwrap())
                        .id(egui::Id::new("terminal"))
                        .text_color(scroller_text_color)
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
