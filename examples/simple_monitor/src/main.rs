use std::sync::{Arc, Mutex, mpsc, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use eframe;
mod ui_app;
use ui_app::App;
use mobius_egui::types::{MobiusString, MobiusDeque}; 
use mobius_egui::clear_logger;

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
                        } else {
                            println!("FirstTask failed.\n");
                            logger_text.lock().unwrap().push_str("Processing FirstTask Command (failed).\n");
                        }
                    }
                    Command::SecondTask => {
                        println!("Processing SecondTask");
                        logger_text.lock().unwrap().push_str("Processing SecondTask Command (success).\n");
                        thread::sleep(Duration::from_millis(100));
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

//****************************************************************
// Main Function - Synchronous Operation with Background Thread
//**************************************************************** 
fn main() {
    let (command_sender, command_receiver) = mpsc::channel::<Command>();
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let app = App {
        logger_text: Arc::new(Mutex::new(String::new())),
        command_sender: command_sender.clone(),
    };

    let logger_text = app.logger_text.clone();


    // Implement the backend task processor for the commands
    let shutdown_flag_clone = shutdown_flag.clone();
    thread::spawn(move || process_commands(logger_text, command_receiver, shutdown_flag_clone));

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


}
