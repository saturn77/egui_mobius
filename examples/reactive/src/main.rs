use std::sync::Arc;
use eframe::NativeOptions;
use egui_mobius_reactive::{Value, Derived, SignalRegistry};
use egui_mobius::factory;
use egui_mobius::Signal;

// Reactive Core Example
// =====================
// 
// This example demonstrates the use of the reactive core in `egui_mobius`.
// It shows how to create reactive signals and bind them to UI elements.
// This is the first example demonstrating the new reactive core.
// 
// The ValueExt trait is used to create reactive signals and bind them to UI elements,
// which is in the egui_mobius_reactive crate. Basically it spins up a reactive context
// or runtime that holds the reactive signals via a thread that is constantly monitoring
// the reactive signals for changes. There is a Derived trait that allows for the creation
// of reactive signals that are derived from other reactive signals.
//
// In summary, to use the reactive system : 
// 1. Define a derived type with a closure that defines how the derived value is computed
//    from the dependencies.  Derived::new(&[dependencies], closure)
// 2. Register the derived type with the reactive context.
// 3. Use the derived type in the UI.
// 
// 20 March 2025, James B, <atlantix-eda@proton.me>

//-------------------------------------------------------------
// AppContext with Derived type on doubled
//-------------------------------------------------------------
pub struct AppContext {
    pub count    : Value<i32>,
    pub label    : Value<String>,
    pub doubled  : Derived<i32>,
}

impl AppContext {
    pub fn new() -> Arc<Self> {
        let count = Value::new(0);
        
        // Create derived value that automatically updates when count changes, which in this 
        // case is the "dependency" of the derived value. The Derived value is a reactive signal
        // that is automatically updated when the dependency changes.
        //
        // The Derived value takes a closure that defines how the derived value is computed 
        // from the dependencies. In this case, the derived value is simply the count value 
        // multiplied by 2.
        
        let count_ref = count.clone();
        let doubled = Derived::new(&[count_ref.clone()], move || {
            let val = *count_ref.lock();
            val * 2
        });

        Arc::new(Self {
            count,
            label: Value::new("Count is 0".to_string()),
            doubled,
        })
    }
}

//-------------------------------------------------------------
// Event Enum - trigger reactive signals
//-------------------------------------------------------------
#[derive(Clone, Debug)]
pub enum Event {
    IncrementClicked,
    CountChanged(i32),
    LabelChanged(String),
}
//-------------------------------------------------------------
// AppState - contains SignalRegistry and Derived<T>
//-------------------------------------------------------------
pub struct AppState {
    pub registry : SignalRegistry,
    count        : Value<i32>,
    label        : Value<String>,
    doubled      : Derived<i32>,
    signal       : Signal<Event>,
}

impl AppState {
    pub fn new(registry: SignalRegistry, signal: Signal<Event>) -> Self {
        let count = Value::new(0);
        
        // Create derived value that automatically updates when count changes
        let count_ref = count.clone();
        let doubled = Derived::new(&[count_ref.clone()], move || {
            let val = *count_ref.lock();
            val * 2
        });
        
        // Create label that updates when count changes
        let label = Value::new("Click to increment".to_string());

        // Register values with the registry
        registry.register_signal(Arc::new(count.clone()));
        registry.register_signal(Arc::new(doubled.clone()));
        
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
            let count = *self.count.lock();
            ui.label(format!("Count: {}", count));

            // Display the doubled value
            let doubled = self.doubled.get();
            ui.label(format!("Doubled: {}", doubled));

            // Button to increment count
            if ui.button(self.label.lock().as_str()).clicked() {
                let new_count = *self.count.lock() + 1;
                self.count.set(new_count);
                if let Err(e) = self.signal.send(Event::IncrementClicked) {
                    eprintln!("Failed to send increment event: {}", e);
                } 
            } 
        }); 
    } 
} 

//-------------------------------------------------------------
// Main with connections of reactive components
//-------------------------------------------------------------
fn main() -> eframe::Result<()> {

    let (event_signal, _event_slot) = factory::create_signal_slot::<Event>();
    
    eframe::run_native(
        "egui_mobius Reactive Example",
        NativeOptions::default(),
        Box::new(move |cc| {
            let _ctx = cc.egui_ctx.clone();
            
            // Create app state with reactive context
            let registry = SignalRegistry::new();
            let event_signal = Arc::new(event_signal);
            registry.register_signal(event_signal.clone()); // Register the signal
            let app_state = AppState::new(registry, (*event_signal).clone());
            
            Ok(Box::new(app_state))
        }),
    )
}
