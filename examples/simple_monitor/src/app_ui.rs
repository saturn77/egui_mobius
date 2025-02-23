use tokio::task;
use eframe;
use egui;
use crate::{Command, CommandResult};
 
use mobius_egui::mobius_send_command;
use mobius_egui::types::*;

pub struct App {
    pub logger_text     : MobiusString,
    pub command_sender  : MobiusCommandEnque<Command>,
    pub event_receiver  : MobiusEventDeque<CommandResult>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            // add buttons to send commands
            ui.horizontal(|ui| {
                if ui.button("First Task").clicked() {
                    println!("First Task button clicked.");
                    mobius_send_command!(self.command_sender, Command::FirstTask);
                }
                if ui.button("Second Task").clicked() {
                    println!("Second Task button clicked.");
                    mobius_send_command!(self.command_sender, Command::SecondTask);
                }
                if ui.button("Clear Terminal").clicked() {
                    println!("Clear Terminal button clicked.");
                    self.logger_text.lock().unwrap().clear();
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
                        .desired_width(550.)
                        .show(ui);
                });
        });
    }
}