# Signals & Slots 

The use of Signals and Slots allows for sending data between threads in an
application, and is the primary method for achieving the main goal of egui_mobius
which is modularity facilitating design reuse and scalability. 

There is a
factory method for creating a signal/slot pair, often useful for use in main.rs
when the application is setting up the signals and slots for the application. 


## Signal/Slot Example 
Consider the ui_refresh_events example main.rs to see the declaration and use
of a signal/slot pairs.  

```rust
fn main() {
    env_logger::init();

    let messages = Value::new(VecDeque::new());
    let update_needed = Value::new(false);
    let logger_text = Value::new("Welcome to egui_mobius ui_refresh_events example ....\n".to_string());

    let (signal_to_backend, slot_to_backend) = factory::create_signal_slot::<EventType>(1);
    let (slot_on_uiapp, slot_from_backend) = factory::create_signal_slot::<ProcessedType>(1);

    backend_consumer_thread(
        Value::clone(&logger_text),
        Value::clone(&messages),
        Value::clone(&update_needed),
        slot_to_backend,
        slot_on_uiapp.clone(),
    );

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_titlebar_buttons_shown(true)
            .with_min_inner_size((650.0, 700.0))
            .with_resizable(true)
            .with_max_inner_size((650.0, 700.0)),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "egui_mobius - UI Refresh Events Example",
        options,
        Box::new(|_cc| Ok(Box::new(UiApp::new(
            signal_to_backend,
            slot_from_backend,
            update_needed,
        )))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }
}
```

### Design Pattern 
Note that the main.rs file length is rather compact. There is a consistent
design pattern in several of the example that : 

The `pattern_mobius_main` typically does:
1. Declares signal/slot pairs for the application in main.rs
2. Declares backend process thread in main.rs
3. Sets options for eframe
4. Runs the actual eframe/egui application


## Slot Core Functionality

The Slot is more sophisticated the Signal, as it sets up it's own thread. The
code below is from the core of egui_mobius, and note the *msg_or_event* syntax
for the argument to the handler. The reason for this is that the Slot can be 
placed on a backend consumer thread, or and UiApplication (UiApp) thread. The
syntax is there to support both use cases, where a Slot may take in an event 
when it's part of the primary background consumer thread, and where it may take
in a processed event on the UiApp side in which case it's receiving a message. 

```rust
impl<T> Slot<T>
where
    T: Send + 'static + Clone,
{
    /// Create a new slot with the given receiver and sequence ID.
    pub fn new(receiver: Receiver<T>, id_sequence : Option<usize>) -> Self {
        Slot {
            receiver: Arc::new(Mutex::new(receiver)),
            sequence: id_sequence.unwrap_or(0),
        }
    }
    /// Start the slot in a separate thread.
    pub fn start<F>(&mut self, mut handler: F)
    where
        F: FnMut(T) + Send + 'static,
    {
        let receiver = Arc::clone(&self.receiver);
        thread::spawn(move || {
            let receiver = receiver.lock().unwrap();
            for msg_or_event in receiver.iter() {
                handler(msg_or_event);
            }
        });
    }
}
```