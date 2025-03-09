# egui_mobius - UI Refresh Events Example

## 🛠️ Running the Example
Ensure you have **Rust and Cargo installed**, then run:
```sh
cargo run -p ui_refresh_events
```
## 🚀 Why This Example Matters
This is currently the **primary example** for `egui_mobius`, showcasing:
- **Best practices for using signals and slots.**
- **Proper separation of frontend and backend logic.**
- **Efficient event-driven UI updates.**
- **Maintainable, scalable code**
- **Optimized logging and UI responsiveness.**

## 📖 Overview
This example, `ui_refresh_events`, demonstrates event-driven UI updates using `egui_mobius`, where signals and slots facilitate communication between the frontend (UI application) and backend (event consumer thread).

It showcases:
- Separation of frontend (UI) and backend (processing logic).
- Bi-directional communication** between UI and backend.
- Flexible Event handling
- Event-based UI updates
- Usage of egui_mobius types** `Value<T >` && `Edge<T>` 
- Ergonomic hanling of `Arc<Mutex<T>>` and improved event detection.
- Efficient logging system that dynamically updates in response to backend events.

## ⚡ Architecture
This example follows a structured **frontend-backend model** using **signals and slots for inter-thread communication.

### 📈 Frontend (`UiApp`)
- Sends events** to the backend via `Signal<EventType>`.
- Receives processed events** via `Slot<ProcessedType>`.
- UI updates only when new events are received
 (avoiding unnecessary redraws).
- Provides user interaction elements:
  - Buttons (`Foo`, `Bar`, `Application Commands`).
  - Slider with real-time updates.
  - ComboBox for selection-based events.
  - Logger system to track event flow.

### 👨‍💻 Backend (`backend_consumer_thread`)
- Receives events from the UI via `Slot<EventType>`.
- Processes events and sends responses back using `Signal<ProcessedType>`.
- Handles application logic:
  - `Foo` and `Bar` events.
  - Slider value updates.
  - Combo box selection tracking.
  - System-level commands (`Clear Logger`, `OS Info`, `Version Info`, `Shutdown`).


## 📂 Code Structure
```
examples/ui_refresh_events/
│── main.rs          # Main entry point defining frontend and backend logic
│── README.md        # This file
│── Cargo.toml       # Dependencies and build configurations
```

##  Key Components
### 🔢 Event System (`EventType` & `ProcessedType`)
Defines **all possible events**, ensuring structured communication.

```rust
#[derive(Debug, Clone)]
enum EventType {
    Foo { id: usize, message: String },
    Bar { id: usize, message: String },
    Slider(usize),
    Combo { _id: usize, message: String },
    ApplicationCommand(String),
}

#[derive(Debug, Clone)]
enum ProcessedType {
    Foo    { id: usize, message: String   },
    Bar    { id: usize, message: String   },
    Slider { message: String              },
    _Combo  { _id: usize, message: String  },    
    _ApplicationCommand { message : String },
}
```

---

### 🛠️ Frontend - `UiApp`
Manages the UI and sends/receives events.

```rust
struct UiApp {
    logger_text         : Value<String>,
    signal_to_backend   : Signal<EventType>,  
    slot_on_uiapp       : Slot<ProcessedType>,  
    update_needed       : Value<bool>,
    slider_value        : Value<Edge<usize>>,
    combo_value         : Value<Edge<String>>,
}
```

---

### 🛠️ Backend - `backend_consumer_thread`
Processes UI events and sends responses.

```rust
fn backend_consumer_thread(
    logger_text         : Value<String>,
    messages            : Value<VecDeque<String>>,
    update_needed       : Value<bool>,
    mut slot            : Slot<EventType>,         // incoming from UiApp
    slot_on_uiapp       : Signal<ProcessedType>,   // outgoing to UiApp
) {
    slot.start({
        move |event| {
            let log_msg = match &event {
                EventType::Foo { id, message } => {
                    format!("Backend processed Foo event [{}]: {}", id, message)
                },
                EventType::Bar { id, message } => {
                    format!("Backend processed Bar event [{}]: {}", id, message)
                },
                EventType::Slider(value) => {
                    format!("Backend processed Slider value: {}", value)
                },
                _ => "Unknown event".to_string(),
            };
            slot_on_uiapp.send(ProcessedType::_ApplicationCommand { message: log_msg.clone() }).unwrap();
        }
    });
}
```

---

## 🛠️ Signals & Slots Workflow
| **Step** | **Action** |
|----------|-----------|
| 1️⃣ User interacts with UI | **Event (e.g., `Slider` change, `Foo` button click) is sent to backend** via `Signal<EventType>` |
| 2️⃣ Backend processes event | **Backend logs event and sends a response to UI** via `Signal<ProcessedType>` |
| 3️⃣ UI receives processed event | **Logger updates, UI refreshes only if needed** |




## 📚 License
This example is part of the `egui_mobius` project and is available under the **MIT License**.


### 🚀 Get Started & Contribute
Contributions are welcome! If you find an issue or have an idea for improvement, feel free to submit a PR.


