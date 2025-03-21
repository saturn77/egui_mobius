//! Thread-safe reactive state management system for egui_mobius.
//! 
//! This crate provides a reactive state management system designed specifically for
//! thread-safe, real-time UI updates in egui applications. It is built on three main
//! concepts:
//! 
//! 1. **Values**: Thread-safe containers for state that can notify listeners of changes
//! 2. **Derived Values**: Automatically updating computed values that depend on other values
//! 3. **Signal Registry**: A system for managing signal-slot connections between components
//! 
//! # Example
//! 
//! ```rust
//! use egui_mobius_reactive::{Value, Derived};
//! 
//! // Create a basic value
//! let count = Value::new(0);
//! 
//! // Create a derived value that automatically updates
//! let doubled = Derived::new(&[count.clone()], move || {
//!     let val = *count.lock().unwrap();
//!     val * 2
//! });
//! 
//! // Update the original value
//! *count.lock().unwrap() = 5;
//! 
//! // The derived value automatically updates
//! assert_eq!(*doubled.get(), 10);
//! ```
//! 
//! # Features
//! 
//! - **Thread Safety**: All state containers are wrapped in `Arc<Mutex<T>>` for safe
//!   concurrent access
//! - **Automatic Updates**: Derived values automatically recompute when their dependencies
//!   change
//! - **Change Notifications**: Values can notify listeners when they change through the
//!   `ValueExt` trait
//! - **Signal-Slot System**: Components can communicate through a type-safe signal-slot
//!   system
//! 
//! # Best Practices
//! 
//! 1. Use `Value<T>` for state that needs to be shared between threads
//! 2. Use `Derived` for computed values that depend on other values
//! 3. Use `SignalRegistry` for loose coupling between components
//! 4. Always handle mutex locks appropriately to avoid deadlocks
//! 5. Consider using the `ValueExt` trait for fine-grained change notifications

pub mod reactive;

// Re-export commonly used types for convenience
pub use reactive::{Value, Derived, SignalRegistry};
