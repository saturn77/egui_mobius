# Reactive Example

This example demonstrates the use of the reactive system in `egui_mobius`. It shows how to create reactive values and computed values that automatically update when their dependencies change.

```bash
cargo run -p reactive
```

## Architecture

### Signal Registry
The reactive system is managed by a `SignalRegistry` which keeps track of all reactive values and their dependencies. This allows for efficient updates when values change.

### Threaded Runtime
The reactive system uses a sophisticated threaded runtime:

1. **Value Change Detection**:
   - Each `Value<T>` can be monitored for changes using the `on_change` method
   - A dedicated background thread is spawned for each monitored value
   - The thread polls the value every 100ms and triggers callbacks when changes are detected

2. **Computed Values**:
   - The `Derived<T>` type represents computed values that depend on other reactive values
   - When dependencies change, the computed value is automatically recalculated
   - Dependencies are tracked through a subscription system

3. **Thread Safety**:
   - All values are protected by `Arc<Mutex<T>>` for safe concurrent access
   - The reactive system ensures thread-safe updates across UI and background threads
   - Changes are propagated safely through the dependency graph

## Example Structure
The entire implementation is contained in a single file (`src/main.rs`) to clearly show:
- How to set up the `SignalRegistry`
- How to create reactive values with `Value<T>`
- How to create computed values with `Derived<T>`
- How changes propagate through the system


---

This example is intentionally simple to focus on the reactive pattern. For a more complex example with multiple files and components, see the `clock_async` example.
