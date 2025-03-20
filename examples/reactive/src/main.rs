use std::sync::Arc;
use egui_mobius::signals::Signal;
use egui_mobius::factory;
use egui_mobius::slot::Slot;
use egui_mobius::types::Value;
use egui_mobius::reactive::{Derived, SignalRegistry};

// Reactive Core Example
// =====================
// 
// This example demonstrates the use of the reactive core in `egui_mobius`.
// It shows how to create reactive signals and bind them to UI elements.
// 
// This is really a proof of concept, focusing on the idea of "reactive" signals.
// There is a context, or reactive context, that holds the reactive signals.
// This context is used to create reactive signals and bind them to UI elements.
// To me, this is implicit chaining, or reactive chaining.
// 
// 20 March 2025, James B, <atlantix-eda@proton.me>

// === Reactive Context === //

pub struct AppContext {
    pub count    : Value<i32>,
    pub label    : Value<String>,
    pub doubled  : Derived<i32>,
}

impl AppContext {
    pub fn new() -> Arc<Self> {
        let count = Value::new(0);
        
        // Create derived value that automatically updates when count changes
        let count_ref = count.clone();
        let doubled = Derived::new(&[count_ref.clone()], move || {
            let val = *count_ref.lock().unwrap();
            val * 2
        });

        Arc::new(Self {
            count,
            label: Value::new("Count is 0".to_string()),
            doubled,
        })
    }
}

// === Event System === //
#[derive(Clone, Debug)]
pub enum Event {
    IncrementClicked,
    CountChanged(i32),
    LabelChanged(String),
}
// === AppState === //
pub struct AppState {
    pub registry: SignalRegistry,
    count: Value<i32>,
    label: Value<String>,
    doubled: Derived<i32>,
    signal: Signal<Event>,
}

impl AppState {
    pub fn new(registry: SignalRegistry, signal: Signal<Event>) -> Self {
        let count = Value::new(0);
        
        // Create derived value that automatically updates when count changes
        let count_ref = count.clone();
        let doubled = Derived::new(&[count_ref.clone()], move || {
            let val = *count_ref.lock().unwrap();
            val * 2
        });
        
        // Create label that updates when count changes
        let label = Value::new("Click to increment".to_string());
        
        Self { 
            registry,
            count,
            label,
            doubled,
            signal 
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Reactive UI with egui_mobius");

            // Display the count
            let count = *self.count.lock().unwrap();
            ui.label(format!("Count: {}", count));

            // Display the doubled value
            let doubled = self.doubled.get();
            ui.label(format!("Doubled: {}", doubled));

            // Button to increment count
            if ui.button(self.label.lock().unwrap().as_str()).clicked() {
                *self.count.lock().unwrap() += 1;
                if let Err(e) = self.signal.send(Event::IncrementClicked) {
                    eprintln!("Failed to send increment event: {}", e);
                } 
            } 
        }); 
    } // End of AppState.update
} // End of AppState

// === Main Entrypoint === //
use eframe::NativeOptions;

fn background_event_thread(mut event_slot: Slot<Event>) {
    std::thread::spawn(move || {
        event_slot.start(move |event| {
            match event {
                Event::IncrementClicked => {
                    // Event already handled in UI
                }
                Event::CountChanged(_) => {
                    // Event already handled in UI
                }
                _ => {}
            }
        });
    });
}

fn main() -> eframe::Result<()> {
    // Create signal/slot pairs for event handling
    let (event_signal, event_slot) = factory::create_signal_slot::<Event>();
    
    // Set up UI
    eframe::run_native(
        "egui_mobius Reactive Example",
        NativeOptions::default(),
        Box::new(move |cc| {
            let _ctx = cc.egui_ctx.clone();
            
            // Create app state with reactive context
            let registry = SignalRegistry::new();
            let app_state = AppState::new(registry, event_signal);
            
            // Start background thread for event processing
            background_event_thread(event_slot);
            
            Ok(Box::new(app_state))
        }),
    )
}
