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


macro_rules! derived {
    ($dep:expr, $power:expr) => {
        Derived::new(&[$dep], move || {
            let val: i32 = $dep.get();
            val.pow($power as u32)
        })
    };
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
    quad         : Derived<i32>,
    fifth        : Derived<i32>,
    sum_derived  : Derived<i32>, 
    signal       : Signal<Event>,
}

impl AppState {
    pub fn new(registry: SignalRegistry, signal: Signal<Event>) -> Self {
        let count = Value::new(0);
        registry.register_signal(Arc::new(count.clone())); // Register count immediately

        // Create derived values
        let count_ref = count.clone();
        let count_ref2 = count.clone();
        let count_ref3 = count.clone();

        // Create derived values using the macro
        let doubled = derived!(count_ref.clone(), 2);
        registry.register_signal(Arc::new(Value::new(doubled.get()))); // Wrap Derived<i32> in Value<i32> and register

        let quad = derived!(count_ref2.clone(), 4);
        registry.register_signal(Arc::new(Value::new(quad.get()))); // Wrap Derived<i32> in Value<i32> and register

        let fifth = derived!(count_ref3.clone(), 5);
        registry.register_signal(Arc::new(Value::new(fifth.get()))); // Wrap Derived<i32> in Value<i32> and register

        // Create a derived value for sum
        let count_for_sum = count.clone();
        let doubled_for_sum = doubled.clone();
        let sum_derived = Derived::new(&[count.clone(), doubled_for_sum.clone().into()], move || {
            let count_val = count_for_sum.get();
            let doubled_val = doubled_for_sum.get(); // Use doubled via Arc
            count_val + doubled_val
        });
        registry.register_signal(Arc::new(Value::new(sum_derived.get()))); // Wrap Derived<i32> in Value<i32> and register

        let label = Value::new("Click to increment".to_string());

        Self {
            registry,
            count,
            label,
            doubled,
            quad,
            fifth,
            sum_derived, // Store sum_derived directly
            signal,
        }
    }
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Reactive UI with egui_mobius");

            // Add spacing between elements
            ui.add_space(20.0);

            // Layout in a horizontal row
            ui.horizontal(|ui| {
                // Display the count
                let count = *self.count.lock();
                ui.label(format!("Count: {}", count));

                // Add spacing between elements
                ui.add_space(20.0);

                // Display the doubled value
                let doubled = self.doubled.get();
                ui.label(format!("Doubled: {}", doubled));

                // Add spacing between elements
                ui.add_space(20.0);

                // Display the quad value
                let quad = self.quad.get();
                ui.label(format!("Quad: {}", quad));

                // Add spacing between elements
                ui.add_space(20.0);

                // Display the fifth value
                let fifth = self.fifth.get();
                ui.label(format!("Fifth: {}", fifth));

                // Add spacing between elements
                ui.add_space(20.0);

                // Display the sum value
                let sum = self.sum_derived.get(); // Corrected field name
                ui.label(format!("Sum: {}", sum));
            });

            // Add spacing between elements
            ui.add_space(20.0);

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
} // end of impl eframe::App for AppState

 

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
