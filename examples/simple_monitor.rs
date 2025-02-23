
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use eframe;
use egui;

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
                    thread::spawn(move || {
                        if let Err(e) = sender.send(Command::FirstTask) {
                            eprintln!("Failed to send FirstTask command: {:?}", e);
                        }
                    });

                    let sender = self.command_sender.clone();
                    thread::spawn(move || {
                        if let Err(e) = sender.send(Command::SecondTask) {
                            eprintln!("Failed to send SecondTask command: {:?}", e);
                        }
                    });
                }
                if ui.button("Second Task").clicked() {
                    println!("Second Task button clicked.");
                    let sender = self.command_sender.clone();
                    thread::spawn(move || {
                        if let Err(e) = sender.send(Command::SecondTask) {
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

#[derive(Debug, Clone)]
pub enum Command {
    FirstTask,
    SecondTask,
}

#[derive(Debug)]
pub enum CommandResult {
    Success(String),
    Failure(String),
}

pub fn process_commands(
    logger_text: Arc<Mutex<String>>,
    command_receiver: mpsc::Receiver<Command>,
    result_sender: mpsc::Sender<CommandResult>,
) {
    let mut local_index: u32 = 0;
    while let Ok(command) = command_receiver.recv() {
        println!("Received command: {:?}", command);

        match command {
            Command::FirstTask => {
                println!("Processing FirstTask...");
                let banner_string = format!("\n**** Processing Iteration {} of GUI Commands.\n", local_index);
                local_index += 1;
                logger_text.lock().unwrap().push_str(&banner_string);

                thread::sleep(Duration::from_millis(100)); // Simulate long task
                if rand::random::<bool>() {
                    println!("FirstTask succeeded.");
                    logger_text.lock().unwrap().push_str("Processing FirstTask Command (success).\n");
                    result_sender.send(CommandResult::Success("First Task completed!".to_string())).unwrap();
                } else {
                    println!("FirstTask failed.\n");
                    logger_text.lock().unwrap().push_str("Processing FirstTask Command (failed).\n");
                    result_sender.send(CommandResult::Failure("First Task failed!".to_string())).unwrap();
                }
            }
            Command::SecondTask => {
                println!("Processing SecondTask");
                logger_text.lock().unwrap().push_str("Processing SecondTask Command (success).\n");
                thread::sleep(Duration::from_millis(100));
                result_sender.send(CommandResult::Success("Second Task completed!".to_string())).unwrap();
            }
        }
    }
}

fn main() {
    let (command_sender, command_receiver) = mpsc::channel::<Command>();
    let (result_sender, result_receiver) = mpsc::channel::<CommandResult>();

    let app = App {
        logger_text: Arc::new(Mutex::new(String::new())),
        command_sender: command_sender.clone(),
        result_receiver: Arc::new(Mutex::new(None)),
    };

    let logger_text = app.logger_text.clone();
    let result_receiver_clone: Arc<Mutex<Option<CommandResult>>> = Arc::clone(&app.result_receiver);

    // Implement the backend task processor for the commands
    thread::spawn(move || process_commands(logger_text, command_receiver, result_sender));

    // Run the app
    if let Err(e) = eframe::run_native(
        "My App",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }

    // Process results
    while let Ok(result) = result_receiver.recv() {
        *result_receiver_clone.lock().unwrap() = Some(result);
    }
}
