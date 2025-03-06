
//-------------------------------------------------------------------------
// Filename : examples/ui_refresh/src/main.rs
// Project  : egui_mobius
// Created  : 05 Mar 2025, James B <atlantix-eda@proton.me>
//-------------------------------------------------------------------------
// Description: 
// This example demonstrates how to use egui_mobius to optimize UI updates
// by using signals and slots to manage message passing between threads.
// This is related to the dispatcher_signals_slots example, but with a 
// focus on UI updates.
//
// Overall, there is one primary producer thread that sends messages 
// to the UI thread. The UI thread updates the UI based on the received 
// messages. The producer thread sends messages to two different slots, 
// which then update the UI. The UI thread only repaints when a new message 
// is received.
//-------------------------------------------------------------------------


use eframe::egui;
use egui_mobius::factory;
use egui_mobius::signals::Signal;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::thread;
use std::time::Duration;

struct MyApp {
    signal1: Signal<String>,
    signal2: Signal<String>,
    messages: Arc<Mutex<VecDeque<String>>>,
    update_needed: Arc<Mutex<bool>>, // Tracks if the UI needs refreshing
}

impl MyApp {
    fn new(
        signal1: Signal<String>,
        signal2: Signal<String>,
        messages: Arc<Mutex<VecDeque<String>>>,
        update_needed: Arc<Mutex<bool>>,
    ) -> Self {
        Self {
            signal1,
            signal2,
            messages,
            update_needed,
        }
    }
}
//-------------------------------------------------------------------------------
// This would normally be "ui_app.rs" but for the sake of the example, it's here
//-------------------------------------------------------------------------------
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut should_repaint = false;

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("egui_mobius - Optimized Dispatcher");

            if ui.button("Send Foo - Egui").clicked() {
                self.signal1.send("Foo - Egui".to_string()).unwrap();
                should_repaint = true;
            }

            if ui.button("Send Bar - Mobius").clicked() {
                self.signal2.send("Bar - Mobius".to_string()).unwrap();
                should_repaint = true;
            }

            ui.separator();

            ui.label("Received Messages:");
            let messages = self.messages.lock().unwrap();
            for msg in messages.iter() {
                ui.label(msg);
            }
        });

        // Only repaint if a new message was received
        let mut update_needed = self.update_needed.lock().unwrap();
        if *update_needed || should_repaint {
            ctx.request_repaint();
            *update_needed = false; // Reset flag after repainting
        }
    }
}

fn main() {
    let messages = Arc::new(Mutex::new(VecDeque::new()));
    let update_needed = Arc::new(Mutex::new(false)); // Shared flag for UI updates

    let messages_clone = Arc::clone(&messages);
    let update_needed_clone = Arc::clone(&update_needed);

    // Create egui_mobius Signals and Slots
    let (signal1, mut slot1) = factory::create_signal_slot::<String>(1);
    let (signal2, mut slot2) = factory::create_signal_slot::<String>(2);

    // Producer thread: updates messages & triggers UI repaint
    thread::spawn(move || {
        slot1.start({
            let messages_clone = Arc::clone(&messages_clone);
            let update_needed_clone = Arc::clone(&update_needed_clone);
            move |msg| {
                let mut queue = messages_clone.lock().unwrap();
                queue.push_back(format!("Handler 1 received: {}", msg));
                *update_needed_clone.lock().unwrap() = true; // Mark UI update required
            }
        });

        slot2.start({
            let messages_clone = Arc::clone(&messages_clone);
            let update_needed_clone = Arc::clone(&update_needed_clone);
            move |msg| {
                let mut queue = messages_clone.lock().unwrap();
                queue.push_back(format!("Handler 2 received: {}", msg));
                *update_needed_clone.lock().unwrap() = true; // Mark UI update required
            }
        });

        loop {
            thread::sleep(Duration::from_millis(100)); // Simulate processing delay
        }
    });

    // Run the UI with egui
    let options = eframe::NativeOptions::default();

    // Run the app
    if let Err(e) = eframe::run_native(
        "egui_mobius - Optimized UI Updates",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new(signal1, signal2, messages, update_needed)))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }

}
