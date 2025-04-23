//! Reactive state management system for thread-safe, real-time UI updates.
//!
//! This module provides a reactive system that enables automatic UI updates when state changes,
//! with built-in thread safety and change detection. It's particularly useful for:
//! 
//! - Managing state that needs to be shared between UI and background threads
//! - Creating computed values that automatically update when their dependencies change
//! - Ensuring thread-safe access to shared state
//! - Building responsive UIs that react to state changes in real-time
//!
//! # Architecture
//!
//! The reactive system consists of three main components:
//!
//! 1. `Dynamic<T>` - A thread-safe container for values that can be monitored for changes
//! 2. `Derived<T>` - Computed values that automatically update when their dependencies change
//! 3. `SignalRegistry` - A registry that manages reactive values and their dependencies
//!
//! # Example
//!
//! ```rust
//! use std::sync::Arc;
//! use egui_mobius_reactive::{Dynamic, Derived, SignalRegistry};
//!
//! // Create a registry to manage reactive values
//! let registry = SignalRegistry::new();
//!
//! // Create a value that can be monitored for changes
//! let count = Dynamic::new(0);
//!
//! // Create a derived value that depends on count
//! let count_for_compute = count.clone();
//! 
//! let doubled = Derived::new(&[Arc::new(count.clone())], move || {
//!     let val = *count_for_compute.lock();
//!     val * 2
//! });
//!
//! // Register the values with the registry
//! registry.register_named_signal("count", Arc::new(count.clone()));
//! registry.register_named_signal("doubled", Arc::new(doubled.clone()));
//!
//! // Values automatically update when dependencies change
//! assert_eq!(doubled.get(), 0);
//! count.set(5);
//! std::thread::sleep(std::time::Duration::from_millis(200));
//! assert_eq!(doubled.get(), 10);
//! ```
//!
//! # Thread Safety
//!
//! All values in the reactive system are protected by `Arc<Mutex<T>>` for safe concurrent access.
//! The system spawns dedicated background threads to monitor for changes and update derived values.
//!
//! # Performance Considerations
//!
//! - Change detection uses a polling approach with a 100ms interval
//! - Consider using `parking_lot::Mutex` instead of `std::sync::Mutex` for better performance
//! - Derived values are only recomputed when their dependencies actually change
pub mod prelude;
pub mod registry;
pub mod dynamic;
pub mod derived;
pub mod core; 
pub mod widgets;
pub mod reactive_math;
pub mod reactive_state;






