# egui_mobius_reactive

A thread-safe reactive state management system for egui_mobius applications.

## Features

- Thread-safe reactive values using `Value<T>`
- Computed values with automatic dependency tracking using `Derived<T>`
- Signal registry for proper lifecycle management
- Change notification system with dedicated background threads
- Safe concurrent access patterns

## Architecture

The reactive system uses a sophisticated threaded runtime:

1. **Value Change Detection**:
   - Each `Value<T>` can be monitored for changes using `on_change`
   - A dedicated background thread is spawned for each monitored value
   - Changes trigger registered callbacks

2. **Computed Values**:
   - `Derived<T>` represents values computed from other reactive values
   - Dependencies are tracked automatically
   - Updates happen automatically when dependencies change

3. **Thread Safety**:
   - All values are protected by `Arc<Mutex<T>>`
   - Changes are propagated safely through the dependency graph
   - The `SignalRegistry` ensures proper lifecycle management

## Usage

```rust
use egui_mobius_reactive::{Value, Derived, SignalRegistry};

// Create a signal registry to manage reactive values
let registry = SignalRegistry::new();

// Create a reactive value
let count = Value::new(0);

// Create a computed value that depends on count
let doubled = Derived::new(&[count.clone()], move || {
    let val = *count.lock();
    val * 2
});

// Register values with the registry
registry.register_signal(Arc::new(count.clone()));
registry.register_signal(Arc::new(doubled.clone()));

// Update values using set() to ensure proper change notification
count.set(5);
```

For a complete example, see the `reactive` example in the main repository.
