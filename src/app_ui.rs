use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task;
use eframe;
use egui;
use crate::{Command, CommandResult};

pub struct App {
    pub logger_text: Arc<Mutex<String>>,
    pub command_sender: mpsc::Sender<Command>,
    pub result_receiver: Arc<Mutex<Option<CommandResult>>>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            // add buttons to send commands
            ui.horizontal(|ui| {
                if ui.button("First Task").clicked() {
                    println!("First Task button clicked.");
                    let sender = self.command_sender.clone();
                    task::spawn(async move {
                        if let Err(e) = sender.send(Command::FirstTask).await {
                            eprintln!("Failed to send FirstTask command: {:?}", e);
                        }
                    });

                    let sender = self.command_sender.clone();
                    task::spawn(async move {
                        if let Err(e) = sender.send(Command::SecondTask).await {
                            eprintln!("Failed to send SecondTask command: {:?}", e);
                        }
                    });
                }
                if ui.button("Second Task").clicked() {
                    println!("Second Task button clicked.");
                    let sender = self.command_sender.clone();
                    task::spawn(async move {
                        if let Err(e) = sender.send(Command::SecondTask).await {
                            eprintln!("Failed to send SecondTask command: {:?}", e);
                        }
                    });
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