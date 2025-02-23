#[allow(dead_code)]
mod app_ui;

use app_ui::App;
use mobius_egui::types::{MobiusString, MobiusCommandDeque, MobiusEventEnque};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use eframe;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Command {
    FirstTask,
    SecondTask,
    ClearTerminal,
}

#[derive(Debug)]
pub enum CommandResult {
    Success(String),
    Failure(String),
}

fn process_commands(
    logger_text       : MobiusString,
    command_receiver  : MobiusCommandDeque<Command>,
    event_sender      : MobiusEventEnque<CommandResult>,
) {
    loop {
        match command_receiver.try_recv() {
            Ok(processed_command) => {
                println!("Processing command: {:?}", processed_command);
                let event_response_string: String;
                match processed_command {
                    Command::FirstTask => {
                        println!("Processing FirstTask...");
                        let banner_string = format!("\n**** Processing Iteration {} of GUI Commands.\n", 1);
                        if let Ok(mut logger) = logger_text.lock() {
                            logger.push_str(&banner_string);
                            logger.push_str("Processing FirstTask Command (success).\n");
                        }
                        event_response_string = "First Task completed!".to_string();
                        println!("FirstTask succeeded.");
                    }
                    Command::SecondTask => {
                        println!("Processing SecondTask");
                        if let Ok(mut logger) = logger_text.lock() {
                            logger.push_str("Processing SecondTask Command (success).\n");
                        }
                        event_response_string = "Second Task completed!".to_string();
                    }
                    Command::ClearTerminal => {
                        println!("Clearing Terminal...");
                        if let Ok(mut logger) = logger_text.lock() {
                            logger.clear();
                        }
                        event_response_string = "Terminal cleared!".to_string();
                        println!("Terminal cleared.");
                    }
                }
                // async send the event response, after launching a tokio runtime
                let _ = tokio::runtime::Runtime::new().unwrap().block_on(async {
                    event_sender.send(CommandResult::Success(event_response_string)).await
                });
            }
            Err(crossbeam_channel::TryRecvError::Empty) => {
                // No command to process, continue the loop
                std::thread::sleep(std::time::Duration::from_millis(100)); // Add a small delay to avoid busy-waiting
            }
            Err(crossbeam_channel::TryRecvError::Disconnected) => {
                println!("Command receiver disconnected");
                break;
            }
        }
    }
}

fn recreate_channels(
    app: &mut App,
) -> (crossbeam_channel::Sender<Command>, crossbeam_channel::Receiver<Command>, mpsc::Sender<CommandResult>, mpsc::Receiver<CommandResult>) {
    let (command_sender, command_receiver) = crossbeam_channel::bounded(0);
    let (event_sender, event_receiver) = mpsc::channel::<CommandResult>(64);

    app.command_sender = command_sender.clone();
    app.event_receiver = event_receiver;

    (command_sender, command_receiver, event_sender, event_receiver)
}

#[tokio::main]
async fn main() {
    let (command_sender, command_receiver) = crossbeam_channel::bounded(64); 
    let (event_sender, event_receiver) = mpsc::channel::<CommandResult>(256); // Increased capacity

    let mut app = App {
        logger_text: Arc::new(Mutex::new(String::new())),
        command_sender,
        event_receiver,
    };

    let logger_text = app.logger_text.clone();

    // Implement the backend task processor for the commands, using std::thread::spawn
    let (command_sender, command_receiver, event_sender, _event_receiver) = recreate_channels(&mut app);
    //std::thread::spawn(move || {
        process_commands(logger_text.clone(), command_receiver.clone(), event_sender.clone());
    //});

    // Run the app
    if let Err(e) = eframe::run_native(
        "My App",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }
}
