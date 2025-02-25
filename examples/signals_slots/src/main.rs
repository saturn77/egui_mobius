
use mobius_egui::factory;
use std::thread;

// General Notes:
// A relatively simple, but yet very important, example of using signals and slots
// to communicate between different parts of a program.
// This example demonstrates how to create a signal and slot pair using the factory function,
// and how to send commands from the signal to the slot.
// The slot is started with a handler function that will process the commands sent from the signal.
// The commands are sent using the send and send_multiple methods of the signal.
// The handler function simply prints the received commands to the console.
// The example also includes a delay to give some time for the commands to be processed.



fn main() {
    // Create a signal and slot via the mobius_egui factory function
    let (signal, slot) = factory::create_signal_slot::<String>();

    // Define a handler function for the slot
    // Note : since the handler takes Fn() as input, it can be a closure or a function
    // The handler function will be called whenever a command is sent to the slot, but the 
    // caller of the handler function is the slot itself.
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