# `clock_async` 

🕒 **Interactive Clock Example with Async Events for `egui_mobius`**

```bash
cargo run --example clock_async
```

This example demonstrates a rich interactive clock application using [`egui_mobius`](https://github.com/saturn77/egui_mobius), showcasing the seamless integration of synchronous UI updates with asynchronous event processing. It features persistent state management, real-time updates, and a beautiful two-column event logging system.

### ✨ Key Features

- ✅ Hybrid Sync/Async Architecture
  - Synchronous UI updates for smooth interaction
  - Asynchronous event processing via `AsyncDispatcher`
  - Background clock updates using `tokio::Runtime`

- ✅ JSON Configuration System
  - Persists UI preferences (12h/24h format)
  - Maintains slider values and combo selections
  - Preserves custom color settings

- ✅ Rich Event Logging
  - Two-column layout for clock and UI events
  - Color-coded event categories
  - Configurable color schemes
  - Source filtering (clock/UI events)

---

## 🧠 Architecture: Event Flow & State Management

The example demonstrates three core `egui_mobius` patterns:

### 1. Clock Updates (Background Thread → UI)
```
Signal<ClockMessage>
   ↓
background_generator_thread
   ↓
AppState.current_time
   ↓
UI refreshes with new time
```

### 2. UI Events (User Input → Async Processing)
```
UI interaction
   ↓
Signal<Event>
   ↓
AsyncDispatcher (tokio task)
   ↓
Signal<Response>
   ↓
AppState update
   ↓
UI refresh
```

### 3. Configuration Management
```
Runtime Changes
  (AppState)
      ↓
JSON serialization
      ↓
Config storage
      ↓
Automatic reload on startup
```

This architecture ensures:
- Non-blocking UI responsiveness
- Type-safe message passing
- Persistent user preferences
- Clean separation of concerns

---

## 🎨 UI Components

- **Live Clock Display**
  - Toggle between 12h/24h format
  - Real-time updates
  - Configurable update frequency

- **Interactive Controls**
  - Slider with async value processing
  - Radio buttons with persistent state
  - Custom event generation

- **Event Logger**
  - Split-view design (Clock | UI Events)
  - Color-coded entries by type
  - Source filtering
  - Scrollable history

---

## 🔧 Implementation Details

The example showcases several advanced `egui_mobius` features:

- **Signal/Slot System**
  - Type-safe message channels
  - Clean thread communication
  - Automatic cleanup

- **Value<T> for Thread Safety**
  - Thread-safe state management
  - Automatic mutex handling
  - Clean state sharing

- **AsyncDispatcher**
  - Non-blocking event processing
  - Typed event handling
  - Response management

This example serves as a comprehensive demonstration of building interactive, state-persistent applications with `egui_mobius`, combining both synchronous and asynchronous operations in a clean, maintainable architecture.
