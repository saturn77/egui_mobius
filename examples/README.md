# Examples in `egui_mobius`

The highlighted examples are the core projects that are actively maintained and intended to showcase the key features of `egui_mobius`.

## 🚀 Getting Started

To get the most out of these examples, we recommend following this learning path:

1. **Basic Concepts** - Start with `ui_refresh_events`
   - Learn basic event handling and UI updates
   - Understand egui_mobius's widget system
   - See how RequestRepaint works

2. **Core Architecture** - Move to `dashboard`
   - Explore the Dispatcher pattern
   - Learn about signal-slot connections
   - See proper UI/backend separation

3. **Real-world Patterns** - Try `realtime_plot` and `dashboard_async`
   - Handle streaming data updates
   - Integrate async operations
   - Work with background tasks

4. **Advanced Features** - Finally, study `clock_async`
   - See thread-safe state management with Value<T>
   - Understand type-safe message passing
   - Learn about AsyncDispatcher and Tokio integration

## 🔧 Setup Instructions

### Prerequisites
- Rust 2024 edition (latest stable toolchain)
- Cargo package manager
- Git for cloning the repository

### Version Compatibility

These examples are compatible with egui_mobius 0.3.0-alpha.1 and egui 0.31.0. They demonstrate the framework's core architectural features:
- Thread-aware slot system
- Type-safe message passing
- Signal-slot architecture
- Value<T> state management
- Background operations

### Running Examples

1. **Clone Repository**
   ```bash
   # Clone the repository
   git clone https://github.com/saturn77/egui_mobius.git
   cd egui_mobius
   ```
   
   Each example is a standalone crate demonstrating specific architectural features of egui_mobius, from basic event handling to complete thread-aware applications.

2. **Run Examples**
   ```bash
   # Run any example as its own crate
   cargo run -p <example_name>
   
   # For instance:
   cargo run -p ui_refresh_events
   cargo run -p dashboard
   cargo run -p clock_async
   ```

3. **Available Examples**
   - `ui_refresh_events` - Introduction to Qt-inspired signal-slot architecture
   - `dashboard` - Type-safe message passing between UI and background threads
   - `realtime_plot` - Thread-safe state management using Value<T>
   - `dashboard_async` - Clean thread separation for background operations
   - `clock_async` - Thread-aware slot system with hybrid sync/async operation

### Configuration & Features

- **Logging**: Most examples support detailed logging
  ```bash
  # Run with debug logging enabled
  RUST_LOG=debug cargo run -p <example_name>
  ```

- **State Persistence**: Some examples (like `clock_async`) support:
  - RON files for static configuration
  - JSON for runtime state persistence

- **Documentation**: Each example has its own README with:
  - Detailed feature explanations
  - Architecture diagrams (where applicable)
  - Example-specific configuration options

---

## ✅ Highlighted Examples

### `ui_refresh_events`
A beginner-friendly example demonstrating basic UI event handling.

- Perfect starting point for understanding egui_mobius basics
- Shows custom timed and programmatic UI refresh events
- Demonstrates basic widget event handling
- Uses simple RequestRepaint pattern without Dispatcher

---

### `dashboard`
Introduces the core Dispatcher pattern for event handling.

- Demonstrates separation of UI and backend concerns
- Shows how to register slots and handle responses
- Features internal logging of UI events and backend processing
- Good template for building modular applications

---

### `realtime_plot`
Focuses on real-time data visualization.

- Shows how to stream data updates to the UI
- Demonstrates dynamic chart updates
- Useful for monitoring and telemetry applications

---

### `dashboard_async`
Builds on the dashboard example with async capabilities.

- Integrates async tasks using tokio runtime
- Handles long-running background processes
- Perfect for applications with API interactions

---

### `clock_async`
A comprehensive example showcasing egui_mobius's complete architecture.

- Demonstrates thread-safe state management using Value<T> for UI controls
- Shows signal-slot system with true concurrent processing (UI vs clock thread)
- Features type-safe message passing between UI and background operations
- Includes detailed logging panel showing the complete event flow
- Uses AsyncDispatcher for specialized async workloads via Tokio

---

## 🏗️ Key Architectural Features

Through these examples, you'll encounter egui_mobius's core architectural features:

- **Thread-aware Slot System** - Each slot maintains its own thread for true hybrid sync/async operation
- **Type-safe Message Passing** - Clean communication between UI and background threads
- **Signal-Slot Architecture** - Qt-inspired design for event handling
- **Value<T> State Management** - Thread-safe state handling with proper synchronization
- **Background Operations** - Clean thread separation with proper message ordering

## 📝 Notes

- Other, more minimal or legacy examples have been moved to the `deprecated` branch
- Dev/test utilities will be in `examples/dev/`
- Each example progressively demonstrates more architectural features
- All examples follow egui_mobius's best practices for production use

---

Feel free to explore these examples when building your own app or library with `egui_mobius`. They cover a range of patterns and are kept up to date with the latest APIs.
