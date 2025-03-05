// Example usage
use egui_mobius::signals::Signal; 
use egui_mobius::slot::Slot; 
use egui_mobius::factory::{create_signal_slot, Dispatcher};
use std::sync::mpsc; 
use std::thread;
use std::time::Duration;

fn main() {
    // Channel between dispatcher and main thread (final receiver)
    let (dispatcher_tx, dispatcher_rx): (mpsc::Sender<String>, mpsc::Receiver<String>) = mpsc::channel();

    // Create a dispatcher
    let mut dispatcher = Dispatcher::new(dispatcher_tx.clone());

    // Add slots to the dispatcher
    let (signal2, slot2): (Signal<String>, Slot<String>) = create_signal_slot(2);
    let (signal3, slot3): (Signal<String>, Slot<String>) = create_signal_slot(3);

    dispatcher.add_slot(slot2);
    dispatcher.add_slot(slot3);

    // Simulate producers
    let signal2_clone = signal2.clone();
    thread::spawn(move || {
        for j in 0..5 {
            let msg = format!("Producer 2 - Message {}", j);
            println!("Sending: {}", msg);
            signal2_clone.send(msg).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
    });

    let signal3_clone = signal3.clone();
    thread::spawn(move || {
        for j in 0..5 {
            let msg = format!("Producer 3 - Message {}", j);
            println!("Sending: {}", msg);
            signal3_clone.send(msg).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
    });

    // Run the dispatcher in a separate thread
    let dispatcher_handle = thread::spawn(move || {
        dispatcher.run();
    });

    // Receive messages in order
    for received in dispatcher_rx {
        println!("[Main] Received: {}", received);
    }

    // Join dispatcher thread (never-ending loop here, adjust as needed)
    dispatcher_handle.join().unwrap();
}
