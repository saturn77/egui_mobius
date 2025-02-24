use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use mobius_egui::signals::Signal;
use mobius_egui::slot::Slot;
use std::thread;

fn main() {
    let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();

    let signal = Signal::new(tx);
    let slot = Slot::new(rx);

    // Define a handler function for the slot
    let handler = |command: String| {
        println!("Handled command: {}", command);
    };

    // Start the slot with the handler
    slot.start(handler);

    // Send a single command
    if let Err(e) = signal.send("Command 1".to_string()) {
        eprintln!("Error sending command: {}", e);
    }

    // Send multiple commands
    if let Err(e) = signal.send_multiple(vec!["Command 2".to_string(), "Command 3".to_string()]) {
        eprintln!("Error sending commands: {}", e);
    }

    // Give some time for the commands to be processed
    thread::sleep(std::time::Duration::from_secs(1));
}