
//-------------------------------------------------------------------------
// Filename : examples/ui_refresh_events/src/main.rs
// Project  : egui_mobius
// Created  : 05 Mar 2025, James B <atlantix-eda@proton.me>
//-------------------------------------------------------------------------
// Description: 
// This example extends the ui_refresh example to include dynamic event types.
// It demonstrates how to use egui_mobius to optimize UI updates by using
// signals and slots to manage message passing between threads. This is related
// to the dispatcher_signals_slots example, but with a focus on UI updates.
//
// The example shows how to create a dynamic event type that can be used to
// send different types of messages to the UI thread. The UI thread updates
// the UI based on the received messages. The producer thread (effectivelye
// the code where "ui_app.rs" would be, sends messages across a single 
// Signal/Slot pair, which then the consumer thread consumes and then 
// updates the UI. The UI thread only repaints when a new message is received.
//
//-------------------------------------------------------------------------
use eframe::egui;
use egui_mobius::factory;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use log::{info, warn}; // Logging framework
use env_logger; // Logger initialization

// Define a dynamic event type - these could be any type of event
// These are important for the producer thread to send messages to the UI
// in an compact, ergonomic way.
#[derive(Debug, Clone)]
enum EventType {
    Foo { id: usize, message: String },
    Bar { id: usize, message: String },
    Custom(String),
}

struct MyApp {
    signal: Signal<EventType>,
    messages: Arc<Mutex<VecDeque<String>>>,
    update_needed: Arc<Mutex<bool>>, // Tracks if the UI needs refreshing
}

impl MyApp {
    fn new(
        signal: Signal<EventType>,
        messages: Arc<Mutex<VecDeque<String>>>,
        update_needed: Arc<Mutex<bool>>,
    ) -> Self {
        Self {
            signal,
            messages,
            update_needed,
        }
    }
}
//----------------------------------------------------------------------------
// This would normally be "ui_app.rs" but for the sake of the example, it's here
//----------------------------------------------------------------------------
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut should_repaint = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui_mobius - Logging System");

            if ui.button("Send Foo Event").clicked() {
                self.signal.send(EventType::Foo { id: 1, message: "Foo - Egui".to_string() }).unwrap();
                should_repaint = true;
            }

            if ui.button("Send Bar Event").clicked() {
                self.signal.send(EventType::Bar { id: 2, message: "Bar - Mobius".to_string() }).unwrap();
                should_repaint = true;
            }

            if ui.button("Send Custom Event").clicked() {
                self.signal.send(EventType::Custom("User-defined event triggered!".to_string())).unwrap();
                should_repaint = true;
            }

            ui.separator();
            ui.label("Received Messages:");
            let messages = self.messages.lock().unwrap();
            for msg in messages.iter() {
                ui.label(msg);
            }
        });

        // Only repaint if a new event was received
        let mut update_needed = self.update_needed.lock().unwrap();
        if *update_needed || should_repaint {
            ctx.request_repaint();
            *update_needed = false; // Reset flag after repainting
        }
    }
}

//-------------------------------------------------------------------------
// Normally this would be backend.rs, but for the sake of the example, it's here
//-------------------------------------------------------------------------
// Consumer thread that processes events and logs them - this could also be 
// called something like "event_handler" or "event_processor".
fn consumer_thread(messages: Arc<Mutex<VecDeque<String>>>, update_needed: Arc<Mutex<bool>>, mut slot: Slot<EventType>) {
    let thread_name = "consumer_thread";
    thread::Builder::new()
        .name(thread_name.to_string())
        .spawn(move || {
            slot.start({
                let messages_clone = Arc::clone(&messages);
                let update_needed_clone = Arc::clone(&update_needed);
                move |event| {
                    let mut queue = messages_clone.lock().unwrap();
                    
                    match event {
                        EventType::Foo { id, message } => {
                            let log_msg = format!("Handler {} processed Foo event: {}", id, message);
                            queue.push_back(log_msg.clone());
                            info!("{}", log_msg); // Log the event
                        }
                        EventType::Bar { id, message } => {
                            let log_msg = format!("Handler {} processed Bar event: {}", id, message);
                            queue.push_back(log_msg.clone());
                            warn!("{}", log_msg); // Log with a warning level
                        }
                        EventType::Custom(msg) => {
                            let log_msg = format!("Custom event processed: {}", msg);
                            queue.push_back(log_msg.clone());
                            info!("{}", log_msg);
                        }
                    }

                    *update_needed_clone.lock().unwrap() = true; // Mark UI update required
                }
            });

            loop {
                thread::sleep(Duration::from_millis(100)); // Simulate processing delay
            }
        })
        .expect("Failed to spawn consumer thread");
}

//-------------------------------------------------------------------------
// Main function, which would normally be by itself in main.rs
//-------------------------------------------------------------------------
fn main() {
    // Initialize logging
    env_logger::init();

    let messages = Arc::new(Mutex::new(VecDeque::new()));
    let update_needed = Arc::new(Mutex::new(false)); // Shared flag for UI updates

    let (signal, slot) = factory::create_signal_slot::<EventType>(1);

    // Start the consumer thread
    consumer_thread(Arc::clone(&messages), Arc::clone(&update_needed), slot);

    // Run the UI with egui
    let options = eframe::NativeOptions::default();

    // Run the app
    if let Err(e) = eframe::run_native(
        "egui_mobius - Dynamic Events (extending ui_refresh) with Logging!",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new(signal, messages, update_needed)))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }
}
