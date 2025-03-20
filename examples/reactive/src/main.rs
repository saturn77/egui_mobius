use std::sync::{Arc, Mutex};
use egui_mobius::signals::Signal;
use egui_mobius::factory;
use egui_mobius::slot::Slot;
use egui_mobius::types::Value;



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

// === Reactive Core === //
type Callback = Box<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub struct SignalValue<T: Clone> {
    value: Arc<Mutex<T>>,
    subscribers: Arc<Mutex<Vec<Callback>>>,
}

impl<T: Clone + std::fmt::Debug> std::fmt::Debug for SignalValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalValue")
            .field("value", &self.value)
            .field("subscribers_count", &self.subscribers.lock().unwrap().len())
            .finish()
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> SignalValue<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(Mutex::new(initial)),
            subscribers: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn get(&self) -> T {
        self.value.lock().unwrap().clone()
    }

    pub fn set(&self, new: T) {
        let mut value = self.value.lock().unwrap();
        if *value != new {
            *value = new;
            self.notify();
        }
    }

    pub fn subscribe(&self, cb: Callback) {
        self.subscribers.lock().unwrap().push(cb);
    }

    fn notify(&self) {
        for cb in self.subscribers.lock().unwrap().iter() {
            cb();
        }
    }
}

// === Derived Signal === //

pub struct Derived<T: Clone> {
    value: Arc<Mutex<T>>,
    deps: Vec<Box<dyn Fn() -> T + Send + Sync>>,
}

impl<T: Clone + 'static + Send> Derived<T> {
    pub fn new<F>(compute: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let initial = compute();
        Self {
            value: Arc::new(Mutex::new(initial)),
            deps: vec![Box::new(compute)],
        }
    }

    pub fn get(&self) -> T {
        let mut value = self.value.lock().unwrap();
        // Recompute value using all dependencies
        if !self.deps.is_empty() {
            *value = self.deps[0]();
        }
        value.clone()
    }
}

// === Declarative Macro === //

#[macro_export]
macro_rules! signal {
    ($name:ident : $ty:ty = $val:expr) => {
        let $name = SignalValue::<$ty>::new($val);
    };
}

// === Reactive Context === //

pub struct ReactiveCtx {
    pub count: Value<i32>,
    pub label: Value<String>,
    pub doubled: Derived<i32>,
}

impl ReactiveCtx {
    pub fn new() -> Arc<Self> {
        let count = Value::new(0);
        
        // Create derived value that automatically updates when count changes
        let count_ref = count.clone();
        let doubled = Derived::new(move || {
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

#[derive(Clone, Debug)]
pub enum MyResponse {
    None,
}

// === AppState === //

pub struct AppState {
    pub ctx: Arc<ReactiveCtx>,
    signal: Signal<Event>,
}

impl AppState {
    pub fn new(ctx: Arc<ReactiveCtx>, signal: Signal<Event>) -> Self {
        Self { ctx, signal }
    }
}



impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Reactive UI with egui_mobius");

            ui.label(format!("Count: {}", *self.ctx.count.lock().unwrap()));
            ui.label(format!("Label: {}", &*self.ctx.label.lock().unwrap()));
            ui.label(format!("Doubled: {}", self.ctx.doubled.get()));

            if ui.button("Increment").clicked() {
                // Send increment event through the dispatcher
                if let Err(e) = self.signal.send(Event::IncrementClicked) {
                    eprintln!("Failed to send increment event: {}", e);
                }
            }
        });
    }
}

fn bind_subscriptions(_ctx: &Arc<ReactiveCtx>, _signal: Signal<Event>) {
    // We don't need subscriptions anymore since we're using Value<T>
    // The background thread handles all state updates
}



// === Main Entrypoint === //
use eframe::NativeOptions;

fn background_event_thread(ctx: Arc<ReactiveCtx>, mut event_slot: Slot<Event>, response_signal: Signal<Event>) {
    std::thread::spawn(move || {
        event_slot.start(move |event| {
            match event {
                Event::IncrementClicked => {
                    // Process in background thread
                    let val = *ctx.count.lock().unwrap();
                    let new_val = val + 1;
                    *ctx.count.lock().unwrap() = new_val;
                    // doubled will update automatically through the Derived binding
                    if let Err(e) = response_signal.send(Event::CountChanged(new_val)) {
                        eprintln!("Failed to send CountChanged event: {}", e);
                    }
                }
                Event::CountChanged(val) => {
                    *ctx.label.lock().unwrap() = format!("Count is now {}", val);
                }
                _ => {}
            }
        });
    });
}

fn main() -> eframe::Result<()> {
    // Create signal/slot pairs for event handling
    let (event_signal, event_slot) = factory::create_signal_slot::<Event>();
    let (response_signal, mut response_slot) = factory::create_signal_slot::<Event>();
    
    // Set up UI
    eframe::run_native(
        "egui_mobius Reactive Example",
        NativeOptions::default(),
        Box::new(move |cc| {
            let _ctx = cc.egui_ctx.clone();
            
            // Create app state with reactive context
            let reactive_ctx = ReactiveCtx::new();
            bind_subscriptions(&reactive_ctx, event_signal.clone());
            let app_state = AppState::new(reactive_ctx, event_signal);
            
            // Start background thread for event processing
            background_event_thread(
                app_state.ctx.clone(),
                event_slot,
                response_signal.clone()
            );
            
            // Set up response handling in UI thread
            let ctx_clone = app_state.ctx.clone();
            response_slot.start(move |event| {
                if let Event::CountChanged(val) = event {
                    ctx_clone.label.set(format!("Count is now {}", val));
                }
            });
            
            Ok(Box::new(app_state))
        }),
    )
}
