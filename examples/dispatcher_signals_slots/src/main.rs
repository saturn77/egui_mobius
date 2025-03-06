// Example usage
use egui_mobius::signals::Signal; 
use egui_mobius::slot::Slot; 
use egui_mobius::factory::{create_signal_slot, Dispatcher};
use std::sync::mpsc; 
use std::thread;
use std::time::Duration;

fn main() {
    let mut dispatcher = Dispatcher::new();

    // Create signal-slot pairs
    let (signal1, slot1) = factory::create_signal_slot::<UiCommand>();
    let (signal2, slot2) = factory::create_signal_slot::<UiCommand>();
    
    // Add slots to dispatcher
    dispatcher.add_slot("ui_command_1", slot1);
    dispatcher.add_slot("ui_command_2", slot2);
    
    // Assign handlers dynamically
    dispatcher.set_handler("ui_command_1", |cmd| println!("Handler 1 received: {:?}", cmd));
    dispatcher.set_handler("ui_command_2", |cmd| println!("Handler 2 received: {:?}", cmd));
    
    // Send commands
    signal1.sender.send(UiCommand::SomeCommand);
    signal2.sender.send(UiCommand::OtherCommand);
    
}
