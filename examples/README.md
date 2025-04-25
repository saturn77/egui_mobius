# Examples in `egui_mobius`

This directory contains examples demonstrating the capabilities and architectural patterns of the `egui_mobius` ecosystem. Each example is designed to showcase specific aspects of the framework.

## 🚀 Learning Path

For the best learning experience, we recommend exploring these examples in the following order:

### 1. Core Paradigms

Start with these to understand the three primary paradigms in egui_mobius:

- **Reactive Pattern**: `reactive` → `reactive_slider` → `dashboard_async` → `clock_reactive`
- **Signal-Slot Pattern**: `ui_refresh_events` → `dashboard` → `clock_async` 
- **Components**: `logger_component`

### 2. Integration Patterns

After understanding the basics, explore how these paradigms work together:

- **Reactive + Async**: `clock_reactive` (shows MobiusRuntime integration)
- **Signals + Async**: `clock_async` (shows AsyncDispatcher with Tokio)
- **Real-time Data Visualization**: `realtime_plot` (shows data streaming patterns)

## 🏗️ Example Architecture

### Signal-Slot Pattern Examples

- **`ui_refresh_events`** - Introduction to signal-slot basics
  - Custom timed and programmatic UI refresh events
  - Simple RequestRepaint pattern

- **`dashboard`** - Core Dispatcher pattern for event handling
  - Demonstrates separation of UI and backend concerns
  - Shows how to register slots and handle responses

- **`clock_async`** - Comprehensive signal-slot architecture
  - Thread-safe state management using Value<T>
  - Shows signal-slot system with true concurrent processing
  - Type-safe message passing between UI and background
  - Uses AsyncDispatcher for Tokio integration

### Reactive Pattern Examples

- **`reactive`** - Basic reactive state management with Dynamic<T>
  - Thread-safe state with automatic UI updates
  - Basic dependency tracking
  - Simple counter patterns

- **`reactive_slider`** - Demonstrates ReactiveWidgetRef
  - Retained-mode style component references
  - Weak references for cleaner composition
  - Reduced Arc<T> cloning pattern

- **`dashboard_async`** - Reactive state with async integration
  - Uses Dynamic<T> for reactive state management
  - Integrates async tasks using MobiusRuntime
  - Demonstrates reactive UI updates with background processing

- **`clock_reactive`** - Complete reactive app with async integration
  - MobiusRuntime for background processing
  - Clean UI/logic separation with reactive state
  - Comprehensive UI with multiple views

### Component Examples

- **`logger_component`** - Demonstrates EventLogger component
  - Sophisticated event logging with customizable styles
  - Thread-safe implementation with signal-slot architecture
  - Multi-level message severity
  - Timestamp filtering and rich text formatting

## 🔧 Running Examples

Each example is a standalone crate that can be run directly:

```bash
# Run any example by name
cargo run -p <example_name>

# For instance:
cargo run -p reactive
cargo run -p logger_component
cargo run -p clock_reactive
```

## 📊 Feature Matrix

| Example | Reactive State | Signal-Slot | Async | Components | Complexity |
|---------|:-------------:|:-----------:|:-----:|:----------:|:----------:|
| reactive | ✅ | - | - | - | 🟢 Basic |
| reactive_slider | ✅ | - | - | - | 🟢 Basic |
| ui_refresh_events | - | ✅ | - | - | 🟢 Basic |
| logger_component | ✅ | ✅ | - | ✅ | 🟡 Moderate |
| dashboard | - | ✅ | - | - | 🟡 Moderate |
| realtime_plot | ✅ | ✅ | - | - | 🟡 Moderate |
| dashboard_async | ✅ | - | ✅ | - | 🟡 Moderate |
| clock_reactive | ✅ | - | ✅ | - | 🔴 Advanced |
| clock_async | ✅ | ✅ | ✅ | - | 🔴 Advanced |

## 🧩 Ecosystem Integration

These examples demonstrate different aspects of the egui_mobius ecosystem:

- `egui_mobius_reactive`: Used in all reactive examples
- `egui_mobius_widgets`: Used in most UI-heavy examples
- `egui_mobius_components`: Used in `logger_component`

## 📝 Configuration & Debugging

- **Logging**: Most examples support detailed logging
  ```bash
  # Run with debug logging enabled
  RUST_LOG=debug cargo run -p <example_name>
  ```

- **State Persistence**: Some examples support:
  - RON files for static configuration
  - JSON for runtime state persistence

## 📚 Documentation

Each example has its own README with:
- Detailed feature explanations
- Architecture notes
- Example-specific configuration options

For a comprehensive introduction to egui_mobius patterns, also check out our [template repository](https://github.com/saturn77/egui_mobius_template).

---

Feel free to explore these examples when building your own application with `egui_mobius`. They cover the full range of architectural patterns and showcase best practices for production use.