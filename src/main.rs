mod app_ui;
mod mobius; // placeholder for the Mobius module framework

use app_ui::App;
use mobius::types::MobiusReceiver;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::task;
use tokio::time::{sleep, Duration};
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

use crate::mobius::types::{MobiusString, MobiusSender};

pub async fn process_commands(
    logger_text: MobiusString,
    mut command_receiver: mpsc::Receiver<Command>,
    result_sender: MobiusSender<CommandResult>,
) {
    let mut local_index: u32 = 0;
    while let Some(command) = command_receiver.recv().await {
        println!("Received command: {:?}", command);

        match command {
            Command::FirstTask => {
                println!("Processing FirstTask...");
                let banner_string = format!("\n**** Processing Iteration {} of GUI Commands.\n", local_index);
                local_index += 1;
                logger_text.lock().unwrap().push_str(&banner_string);

                sleep(Duration::from_millis(100)).await; // Simulate long task
                if rand::random::<bool>() {
                    let mut delay_counter = 0;
                    println!("FirstTask succeeded.");
                    logger_text.lock().unwrap().push_str("Processing FirstTask Command (success).\n");
                    while delay_counter < 10 {
                        delay_counter += 1;
                        sleep(Duration::from_millis(200)).await;
                        *logger_text.lock().unwrap() += &format!(". ");
                    }
                    result_sender.send(CommandResult::Success("First Task completed!".to_string())).await.unwrap();
                } else {
                    println!("FirstTask failed.\n");
                    logger_text.lock().unwrap().push_str("Processing FirstTask Command (failed).\n");
                    result_sender.send(CommandResult::Failure("First Task failed!".to_string())).await.unwrap();
                }
            }
            Command::SecondTask => {
                println!("Processing SecondTask");
                logger_text.lock().unwrap().push_str("Processing SecondTask Command (success).\n");
                sleep(Duration::from_millis(100)).await;
                result_sender.send(CommandResult::Success("Second Task completed!".to_string())).await.unwrap();
            }
        }
    }
}

#[tokio::main]
async fn main() {

    let (command_sender, command_receiver) = mpsc::channel::<Command>(32);
    let (result_sender, mut result_receiver) = mpsc::channel::<CommandResult>(32);



    let app = App {
        logger_text: Arc::new(Mutex::new(String::new())),
        command_sender,
        result_receiver,
    };

    let logger_text = app.logger_text.clone();


    // Implement the backend task processor for the commands
    task::spawn(process_commands(logger_text, command_receiver, result_sender));

    // Run the app
    if let Err(e) = eframe::run_native(
        "My App",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }


}
