use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::Duration;
use eframe;
mod ui_app;
use std::fmt::Display; 

use ui_app::App;
use egui_mobius::clear_logger;
use egui_mobius::factory;
use egui_mobius::types::Value;

use as_command_derive::AsCommand;
use egui_mobius::Signal; 
use std::fmt; 

#[derive(AsCommand, Clone, Debug)]
pub enum UiCommand {
    FirstTask,
    SecondTask,
    ClearTerminal,
    About,
    CascadeFirstSecond(Vec<UiCommand>),
}

impl Display for UiCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


#[derive(Debug)]
pub enum CommandResult {
    Success(String),
    Failure(String),
}

fn handle_command(
    command: UiCommand,
    logger_text: Value<String>,
    local_index: &mut u32,
    shutdown_flag: Arc<AtomicBool>,
) {
    match command {
        UiCommand::FirstTask => {
            println!("Processing FirstTask...");
            let banner_string = format!("\n**** Processing Iteration {} of GUI Commands.\n", local_index);
            *local_index += 1;
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
        UiCommand::SecondTask => {
            println!("Processing SecondTask");
            logger_text.lock().unwrap().push_str("Processing SecondTask Command (success).\n");
            thread::sleep(Duration::from_millis(100));
        }
        UiCommand::ClearTerminal => {
            println!("Clearing Terminal...");
            clear_logger!(logger_text);
        }
        UiCommand::About => {
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
        UiCommand::CascadeFirstSecond(commands) => {
            println!("Processing Multiple Commands...");
            for command in commands {
                handle_command(command, logger_text.clone(), local_index, shutdown_flag.clone());
            }
        }
    }

    // Check the shutdown flag
    if shutdown_flag.load(Ordering::Relaxed) {
        println!("Shutdown flag is set. Exiting handle_command.");
        return;
    }
}

//****************************************************************
// Main Function - Synchronous Operation with Background Thread
//**************************************************************** 
fn main() {
    let (signal, mut slot) = factory::create_signal_slot::<UiCommand>(1);
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    let app = App {
        logger_text: Value::new(String::new()),
        command_sender: signal.sender.clone(),
    };

    let logger_text = app.logger_text.clone();
    let local_index = Arc::new(Mutex::new(0u32));

    // Define a handler function for the slot
    let handler = {
        let shutdown_flag = shutdown_flag.clone();
        let local_index = local_index.clone();
        move |command: UiCommand| {
            let mut local_index = local_index.lock().unwrap();
            handle_command(command, logger_text.clone(), &mut local_index, shutdown_flag.clone());
        }
    };

    // Start the slot with the handler
    slot.start(handler);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size((650.0, 500.0)),
        ..Default::default()
    };


    // Run the app
    if let Err(e) = eframe::run_native(
        "Simple Monitor 2 Demo - Mobius with Egui",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }

    // Set the shutdown flag to true to signal the threads to stop
    shutdown_flag.store(true, Ordering::Relaxed);
}
