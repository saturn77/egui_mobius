# Reactive Example

This example demonstrates the use of the reactive core or "context" in `egui_mobius`.
It shows how to create reactive signals and bind them to UI elements.

The entire implementation is contained in a single file (`src/main.rs`) to clearly show the setup of a reactive context.

## Key Features
- Single-file implementation for clarity
- Reactive context with signal/slot pattern
- Thread-safe state management using `Value<T>`
- Background processing with proper thread separation

## Running
```bash
cargo run -p reactive
```

This example is intentionally simple to focus on the reactive pattern. For a more complex example with multiple files and components, see the `clock_async` example.
