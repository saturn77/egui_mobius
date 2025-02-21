mod app_ui;
mod mobius; // placeholder for the Mobius module framework

use app_ui::App;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use eframe;

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
