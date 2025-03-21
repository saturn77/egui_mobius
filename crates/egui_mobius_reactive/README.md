# egui_mobius_reactive

A thread-safe reactive state management system for egui_mobius applications.

## Overview

The reactive system in `egui_mobius_reactive` provides a powerful way to create reactive UIs with automatic state management. It spins up a reactive context that monitors signals for changes and automatically updates derived values and UI elements.

### Key Components

- `Value<T>`: Thread-safe reactive values
- `Derived<T>`: Computed values that auto-update based on dependencies
- `SignalRegistry`: Central manager for reactive context and signals

## Usage Pattern

1. Define derived types with closures that compute values from dependencies
2. Register values and signals with the SignalRegistry
3. Use the reactive values in your UI

## Complete Example

```rust
use std::sync::Arc;
use egui_mobius_reactive::{Value, Derived, SignalRegistry};

// Define your application state
pub struct AppState {
    pub registry: SignalRegistry,
    count: Value<i32>,
    label: Value<String>,
    doubled: Derived<i32>,
}

impl AppState {
    pub fn new(registry: SignalRegistry) -> Self {
        let count = Value::new(0);
        
        // Create a derived value that auto-updates when count changes
        let count_ref = count.clone();
        let doubled = Derived::new(&[count_ref.clone()], move || {
            let val = *count_ref.lock();
            val * 2
        });
        
        // Create UI label
        let label = Value::new("Click to increment".to_string());

        // Register with SignalRegistry for lifecycle management
        registry.register_signal(Arc::new(count.clone()));
        registry.register_signal(Arc::new(doubled.clone()));
        
        Self { 
            registry,
            count,
            label,
            doubled,
        }
    }

    pub fn increment(&self) {
        let new_count = *self.count.lock() + 1;
        self.count.set(new_count);
        // Doubled value updates automatically!
    }
}

// UI Integration Example
impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Reactive UI Example");

            // Display reactive values
            let count = *self.count.lock();
            ui.label(format!("Count: {}", count));

            let doubled = self.doubled.get();
            ui.label(format!("Doubled: {}", doubled));

            // UI updates trigger reactive changes
            if ui.button(self.label.lock().as_str()).clicked() {
                self.increment();
            }
        });
    }
}

// Application Setup
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Reactive Example",
        Default::default(),
        Box::new(|_cc| {
            // Initialize reactive system
            let registry = SignalRegistry::new();
            let app = AppState::new(registry);
            Box::new(app)
        }),
    )
}
```

### Key Features Demonstrated

1. **Automatic Updates**:
   - `doubled` automatically updates when `count` changes
   - No manual synchronization needed

2. **Thread Safety**:
   - All values are protected by Arc<Mutex<T>>
   - Safe to use across UI and background threads

3. **Lifecycle Management**:
   - SignalRegistry handles registration and cleanup
   - Prevents memory leaks from orphaned signals

4. **UI Integration**:
   - Reactive values seamlessly integrate with egui
   - UI updates trigger reactive updates automatically

## Best Practices

1. **Signal Registration**:
   - Always register values that other components depend on
   - Register derived values that need lifecycle management

2. **State Organization**:
   - Keep SignalRegistry at the app level
   - Group related values in meaningful structs

3. **Thread Safety**:
   - Use Value::lock() for thread-safe access
   - Clone values before moving into closures

4. **Cleanup**:
   - Let SignalRegistry handle automatic cleanup
   - Use manual cleanup only when necessary

## Documentation Example

```rust
use egui_mobius_reactive::{Value, Derived};

// Create a basic value
let count = Value::new(0);

// Create a derived value that automatically updates
let doubled = Derived::new(&[count.clone()], move || {
    let val = *count.lock().unwrap();
    val * 2
});

// Update the original value
*count.lock().unwrap() = 5;

// The derived value automatically updates
assert_eq!(*doubled.get(), 10);
```

For more examples and patterns, see the `reactive` example in the main repository.
