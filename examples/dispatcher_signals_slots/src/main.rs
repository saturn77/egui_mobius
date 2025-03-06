// Example usage

use egui_mobius::factory;
use egui_mobius::factory::Dispatcher;


fn main() {

    println!("\n**** Dispatcher Signals and Slots Example -- egui_mobius framework. \n");

    let mut dispatcher = Dispatcher::new();

    // Create signal-slot pairs
    let (signal1, slot1) = factory::create_signal_slot::<String>(1);
    let (signal2, slot2) = factory::create_signal_slot::<String>(2);
    
    // Add slots to dispatcher
    dispatcher.add_slot("ui_command_1", slot1);
    dispatcher.add_slot("ui_command_2", slot2);
    
    // Assign handlers dynamically !! 
    dispatcher.set_handler("ui_command_1", |cmd| println!("Handler 1 received: {:?}", cmd));
    dispatcher.set_handler("ui_command_2", |cmd| println!("Handler 2 received: {:?}", cmd));
    
    // Send commands
    if let Err(e) = signal1.sender.send("Foo - Egui".to_string()) {
        eprintln!("Failed to send signal1: {:?}", e);
    }
    
    if let Err(e) = signal2.sender.send("Bar - Mobius".to_string()) {
        eprintln!("Failed to send signal2: {:?}", e);
    }

    

    // sleep for enough time for the threads to finish
    std::thread::sleep(std::time::Duration::from_secs(1));

    println!("\n**** Dispatcher Signals and Slots Example - End \n");
}
