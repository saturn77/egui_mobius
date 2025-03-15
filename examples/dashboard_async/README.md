# `dashboard_async` 

🚀 **Asynchronous Dashboard Example for `egui_mobius`**

```bash
cargo run --example dashboard_async
```

This example builds a fully asynchronous dashboard UI using [`egui_mobius`](https://github.com/saturn77/egui_mobius), `egui`, and `eframe`. It fetches real-time cryptocurrency prices from Kraken and displays them in a responsive, state-driven GUI.

It introduces a powerful architectural component to the `egui_mobius` framework: the **`AsyncDispatcher`** — responsible for:
- Receiving typed `Event` messages
- Assigning async tasks to a `tokio::Runtime`
- Returning results via `Signal<Processed>`


### ✨ Key Features

- ✅ `AsyncDispatcher<Event, Processed>` for clean, typed task handling
- ✅ Background async tasks with `tokio::Runtime`
- ✅ Live price fetching for multiple cryptocurrencies
- ✅ Shared `AppState` using `Value<T>` for thread-safe UI state
- ✅ Live, color-coded log panel using `egui::ScrollArea`
- ✅ No need for `#[tokio::main]` — backend stays async, frontend stays sync

---

## 🧠 Architecture: Signal → Async → Signal → State Update → UI

This example demonstrates a core `egui_mobius` pattern:

```
Signal<Event>
   ↓
AsyncDispatcher (tokio task)
   ↓
Signal<Processed>
   ↓
AppState.update(msg: Processed)
   ↓
UI refreshes with new state
```

This clean separation ensures:
- Predictable message-driven state updates
- No blocking UI logic
- Easily testable and extensible system

---

## 🧱 System Overview

### **UI Layer (`UiMainWindow`)**
- Sends `Event` to `AsyncDispatcher` via `Signal<Event>`
- Listens for `Processed` updates via `Slot<Processed>`
- Calls `AppState::update(...)` to apply changes

### **State Layer (`AppState`)**
- Owns all shared UI state (prices, logs, loading status)
- Implements `Updatable<Processed>` trait
- Processes backend messages and updates the UI

### **Backend Dispatcher (`AsyncDispatcher`)**
- Listens for `Event` messages on a `Slot<Event>`
- Spawns async tasks on an internal `tokio::Runtime`
- Emits `Processed` results back to the UI via `Signal<Processed>`

---

## 🖼️ Screenshot

Click a button to trigger a background fetch. The Dispatcher handles the async request and updates the UI log automatically:

![egui_mobius dashboard architecture](../../assets/dashboard_async.png)

---

## 📚 Summary

This is a **flagship example** for the `egui_mobius` framework. It proves:
- Async and sync layers can coexist cleanly
- Message-based UI logic is ergonomic and scalable
- `egui_mobius` is ready for real-world async GUI applications

> ✅ Clean architecture: **Signal → Async Task → Signal Back → AppState Update → UI Reaction**

Feel free to build on top of this as the foundation for dashboards, data explorers, or async-driven tools!
