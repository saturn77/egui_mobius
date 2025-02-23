use std::sync::{Arc, Mutex, mpsc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use eframe;
use egui;

use mobius_egui::types::{MobiusString, MobiusEnque, MobiusDeque}; 

use mobius_egui::{mobius_send_command, clear_logger};

pub struct App {
    pub logger_text     : MobiusString,
    pub command_sender  : MobiusEnque<Command>,
    pub result_receiver : Arc<Mutex<Option<CommandResult>>>,
    pub shutdown_flag   : Arc<AtomicBool>,
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
                    mobius_send_command!(self.command_sender, Command::ClearTerminal);
                }
                if ui.button("Send Multiple Commands").clicked() {
                    println!("Send Multiple Commands button clicked.");
                    let commands = vec![Command::FirstTask, Command::SecondTask];
                    mobius_send_command!(self.command_sender, commands, multiple);
                }
                if ui.button("About").clicked() {
                    println!("About button clicked.");
                    mobius_send_command!(self.command_sender, Command::About);
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
    ClearTerminal,
    About,
}

#[derive(Debug)]
pub enum CommandResult {
    Success(String),
    Failure(String),
}

pub fn process_commands(
    logger_text       : MobiusString,
    command_receiver  : MobiusDeque<Command>,
    result_sender     : MobiusEnque<CommandResult>,
    shutdown_flag     : Arc<AtomicBool>,
) {
    let mut local_index: u32 = 0;
    while !shutdown_flag.load(Ordering::Relaxed) {
        match command_receiver.recv_timeout(Duration::from_millis(100)) {
            Ok(command) => {
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
                    Command::ClearTerminal => {
                        println!("Clearing Terminal...");
                        clear_logger!(logger_text);
                    }
                    Command::About => {
                        println!("Displaying About...");
                        clear_logger!(logger_text); 
                        let mut about_string : String = format!("\n\n*** About - a simple monitor app.\n"); 
                        about_string += "This app demonstrates how to use Mobius with Egui. This app has : \n";
                        about_string += " - Buttons to send commands to the backend.\n";
                        about_string += " - Backend processes the commands and sends results back to the frontend.\n";
                        about_string += " - A terminal window to display the results.\n";
                        about_string += " - Terminal window can be cleared using the 'Clear Terminal' button.\n";
                        about_string += " - The 'About' button displays this message.\n";
                        logger_text.lock().unwrap().push_str(&about_string);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // No command to process, continue the loop
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                println!("Command receiver disconnected");
                break;
            }
        }
    }
    println!("Shutting down process_commands thread.");
}

//**********************************************************
// Main Function - No Tokio Runtime Required
//********************************************************** 
fn main() {
    let (command_sender, command_receiver) = mpsc::channel::<Command>();
    let (result_sender, result_receiver) = mpsc::channel::<CommandResult>();
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let app = App {
        logger_text: Arc::new(Mutex::new(String::new())),
        command_sender: command_sender.clone(),
        result_receiver: Arc::new(Mutex::new(None)),
        shutdown_flag: shutdown_flag.clone(),
    };

    let logger_text = app.logger_text.clone();
    let result_receiver_clone: Arc<Mutex<Option<CommandResult>>> = Arc::clone(&app.result_receiver);

    // Implement the backend task processor for the commands
    let shutdown_flag_clone = shutdown_flag.clone();
    thread::spawn(move || process_commands(logger_text, command_receiver, result_sender, shutdown_flag_clone));

    // Run the app
    if let Err(e) = eframe::run_native(
        "My App",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }

    // Set the shutdown flag to true to signal the threads to stop
    shutdown_flag.store(true, Ordering::Relaxed);

    // Process results
    while let Ok(result) = result_receiver.recv() {
        *result_receiver_clone.lock().unwrap() = Some(result);
    }
}
