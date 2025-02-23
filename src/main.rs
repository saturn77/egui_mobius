#[allow(dead_code)]
mod app_ui;
mod mobius; // placeholder for the Mobius module framework

use app_ui::App;
use mobius::types::{MobiusString, MobiusCommandDeque, MobiusEventEnque};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;


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

fn process_commands(
    logger_text       : MobiusString,
    command_receiver  : MobiusCommandDeque<Command>,
    event_sender      : MobiusEventEnque<CommandResult>,
) {

    if let Ok(processed_command) = command_receiver.try_recv(){
        println!("Processing command: {:?}", processed_command);
        let event_response_string : String;
        match processed_command {
            Command::FirstTask => {
                println!("Processing FirstTask...");
                let banner_string = format!("\n**** Processing Iteration {} of GUI Commands.\n", 1);
                logger_text.lock().unwrap().push_str(&banner_string);
                event_response_string = "First Task completed!".to_string();
                println!("FirstTask succeeded.");
                logger_text.lock().unwrap().push_str("Processing FirstTask Command (success).\n");
                
            }
            Command::SecondTask => {
                println!("Processing SecondTask");
                logger_text.lock().unwrap().push_str("Processing SecondTask Command (success).\n");
                event_response_string = "Second Task completed!".to_string();
                
            }
        }
        // async send the event response
        event_sender.send(CommandResult::Success(event_response_string));
    }
}

#[tokio::main]
async fn main() {

    let (command_sender, command_receiver) = crossbeam_channel::bounded(32); 
    let (event_sender, event_receiver) = mpsc::channel::<CommandResult>(32);



    let app = App {
        logger_text: Arc::new(Mutex::new(String::new())),
        command_sender,
        event_receiver,
    };

    let logger_text = app.logger_text.clone();


    // Implement the backend task processor for the commands, using std::thread::spawn
    std::thread::spawn(move || {
        loop {
            process_commands(logger_text.clone(), command_receiver.clone(), event_sender.clone());
        }
    });

    // Run the app
    if let Err(e) = eframe::run_native(
        "My App",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }

}
