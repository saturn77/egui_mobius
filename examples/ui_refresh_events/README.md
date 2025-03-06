# Dynamic UI Event Handling in `egui_mobius`
### 🚀 **Extending `ui_refresh` with Dynamic Events & Logging**

## **📌 Overview**
The `ui_refresh_events` example demonstrates how to:
- **Dynamically handle different event types** in `egui_mobius` using **signals and slots**.
- **Optimize UI updates**, ensuring efficient repainting **only when new events arrive**.
- **Log event processing using `env_logger`**, displaying logs in **both the console and UI**.

This extends the previous `ui_refresh` example by introducing a **structured event system** and **real-time logging**, making it a powerful demonstration of `egui_mobius`'s capabilities.

---

## **🛠 Features**
✅ **Dynamic Event System:** Uses an `enum EventType` to handle multiple event types.  
✅ **Optimized UI Rendering:** UI updates **only when necessary**, reducing redundant repaints.  
✅ **Thread-Safe Event Processing:** Events are handled in a **separate producer thread**, ensuring responsiveness.  
✅ **Integrated Logging System:** Uses `env_logger` to log events **both in the console and a UI panel**.  
✅ **Decoupled Event Handling:** Uses `egui_mobius` **signals and slots** to send and process events cleanly.

---

## **🌜 How It Works**
### **Defining a Dynamic Event System**
Instead of using raw `String` messages, this example introduces an **`enum EventType`**:
```rust
enum EventType {
    Foo { id: usize, message: String },
    Bar { id: usize, message: String },
    Custom(String),
}
```
This allows **structured event handling**, making the system **scalable and flexible**.

### **Sending Events from the UI**
Button clicks trigger different event types, which are **sent through a signal**:
```rust
if ui.button("Send Foo Event").clicked() {
    self.signal.send(EventType::Foo { id: 1, message: "Foo - Egui".to_string() }).unwrap();
}
```

### **Processing Events in a Separate Thread**
A **consumer thread listens for events** and processes them based on type:
```rust
slot.start(move |event| {
    match event {
        EventType::Foo { id, message } => info!("Handler {} processed Foo event: {}", id, message),
        EventType::Bar { id, message } => warn!("Handler {} processed Bar event: {}", id, message),
        EventType::Custom(msg) => info!("Custom event processed: {}", msg),
    }
});
```
The processed events are **stored and displayed in the UI log panel**.

### **4️⃣ Displaying Logs in the UI**
Instead of keeping a separate queue for logs, this example **uses `env_logger` to capture logs dynamically**.  
The logs are **visible both in the console and an `egui` panel**.

---

## **📺 UI Layout**
| UI Element | Description |
|------------|-------------|
| **Left Panel** | Buttons to trigger different event types (`Foo`, `Bar`, `Custom`) |
| **Center Panel** | Displays the received messages dynamically |
| **Right Panel** | Displays event logs in real-time using `env_logger` |

---

## **🔧 How to Run**
Make sure you have `egui_mobius` installed, then run the example with:
```sh
RUST_LOG=info cargo run --example ui_refresh_events
```
For **verbose debugging**, use:
```sh
RUST_LOG=debug cargo run --example ui_refresh_events
```

---

## **📚 Key Takeaways**
- **`egui_mobius` provides an efficient signal-slot mechanism** for decoupled UI event handling.
- **Dynamic event types (`enum EventType`) make the system scalable** for real-world applications.
- **Logging (`env_logger`) seamlessly integrates into both UI and console**, improving debugging.
- **Optimized UI updates ensure smooth rendering** and prevent unnecessary redraws.

---

## **🔮 Next Steps**
Want to extend this example further? Consider:
1. **Adding timestamps to logs** for better tracking.
2. **Filtering logs dynamically in the UI.**
3. **Exporting logs to a file** for debugging purposes.

---

## **📝 Conclusion**
The `ui_refresh_events` example showcases **a robust event-driven architecture** for `egui_mobius`, **blending signals, structured event handling, and logging into a seamless UI experience**. It serves as a **powerful reference** for building **scalable and efficient UI applications**.

---
🚀 **Happy Coding with `egui_mobius`!** 🎯🔥

