# egui_mobius_macros

🚀 **Procedural Macros for egui_mobius** 🚀

This crate provides a template for future procedural macros that will help reduce boilerplate when working with the `egui_mobius` framework.

## Overview

Currently, this crate serves as a placeholder and template for future derive macros. It provides the basic structure and tooling needed to implement procedural macros that will enhance the `egui_mobius` development experience.

### Planned Features

🔄 **State Management**
- Auto-implement state management traits
- Thread-safe state synchronization
- Value<T> integration

🔌 **Signal/Slot Helpers**
- Automatic signal-slot connections
- Type-safe message passing
- Event routing decorators

🎨 **UI Components**
- Component generation
- Layout helpers
- State binding

⚡ **Event Handling**
- Event type generation
- Handler registration
- Dispatcher integration

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
egui_mobius_macros = { path = "../egui_mobius_macros" }
```

## Development

To implement a new derive macro:

1. Use the template in `src/lib.rs`:
```rust
#[proc_macro_derive(MyMacro)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    // Implementation here
}
```

2. Add tests to verify behavior:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_my_macro() {
        // Test implementation
    }
}
```

3. Document usage and examples

## Contributing

Contributions are welcome! When adding new macros:

- Follow the template structure in `src/lib.rs`
- Include comprehensive documentation
- Add tests for all functionality
- Update this README with usage examples

## License

This project is licensed under the MIT License.

---

🎉 **Ready to enhance egui_mobius with powerful macros!** 🚀
