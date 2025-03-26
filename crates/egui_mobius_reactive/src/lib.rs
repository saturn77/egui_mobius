//! # egui_mobius_reactive
//!
//! `egui_mobius_reactive` simplifies state synchronization and reactivity for GUI applications created with `egui`.
//!
//! Reactive programming is a powerful design pattern essential for building complex, responsive UIs. This crate
//! enables a clean and declarative approach to managing UI state by providing **dynamic** and **derived state objects**
//! that update automatically based on underlying changes. Stop worrying about synchronizing dependent state manually—reactivity is built in.
//!
//! ## Key Features
//!
//! - **Dynamic State (`Dynamic<T>`)**:
//!   Thread-safe, mutable values that automatically notify listeners of updates.
//! - **Derived State (`Derived<T>`)**:
//!   Automatically computed values derived from one or more `Dynamic` objects. Dependents are recomputed only when
//!   their input changes, following reactive programming paradigms (efficient and declarative).
//! - **Signal Registry (`signal_registry`)**:
//!   A centralized system to manage reactive values and their dependencies.
//!   Useful for ensuring values aren't dropped, managing their lifecycle, and debugging or visualizing the reactive graph.
//! - **Integration with `egui`**:
//!   Optimized for use with `egui`, this crate allows seamless management of UI state in interactive applications.
//! - **Thread-Safe Primitives**:
//!   Built on top of `Arc<Mutex<T>>`, ensuring safety when accessing and modifying shared state across threads.
//!
//! ## Getting Started
//!
//! Add the crate to your project with Cargo:
//!
//! ```sh
//! cargo add egui_mobius_reactive
//! ```
//!
//! ## Simple Example: Dynamic and Derived State
//!
//! Here's how you can use `Dynamic` and `Derived` to build reactive state management in your application:
//!
//! ```rust
//! use egui_mobius_reactive::{Dynamic, Derived};
//! use std::sync::Arc;
//!
//! fn main() {
//!     // Define a basic dynamic state variable
//!     let count = Dynamic::new(0);
//!
//!     // Define a derived value that automatically recalculates when `count` changes
//!     let count_clone = count.clone();
//!     let doubled = Derived::new(&[Arc::new(count.clone())], move || {
//!         let val = *count_clone.lock();
//!         val * 2
//!     });
//!
//!     // Mutate the `Dynamic` state
//!     count.set(5);
//!
//!     // Assert the updated, automatically calculated value
//!     assert_eq!(doubled.get(), 10);
//! }
//! ```
//!
//! ## Advanced Example: Using the Signal Registry
//!
//! The `signal_registry` plays a central role in managing reactive values and their dependencies across the system. It ensures that:
//! - Reactive values aren't dropped while they're still needed.
//! - The lifecycle of reactive values is managed effectively.
//! - Debugging and visualization of the reactive graph are streamlined.
//!
//! Here's an example of how to register and use signals in the registry:
//!
//! ```rust
//! use egui_mobius_reactive::{signal_registry, Signal};
//! use std::sync::Arc;
//!
//! fn main() {
//!     // Create a new signal registry
//!     let mut registry = signal_registry();
//!
//!     // Register a named signal for "count"
//!     let count_signal = Signal::new(10);
//!     registry.register_signal("count", count_signal.clone());
//!
//!     // Retrieve and use the signal
//!     let retrieved_signal = registry.get_signal("count").expect("Signal 'count' not found");
//!
//!     // Update the signal
//!     retrieved_signal.set(20);
//!
//!     // Ensure the registry reflects the updated value
//!     assert_eq!(*retrieved_signal.lock(), 20);
//! }
//! ```
//!
//! In this example, the `signal_registry` allows you to centralize the management of reactive state, providing a single point of control and access for reactive values in the ecosystem. This is particularly helpful when building large, complex reactive systems.
//!
//! ## Crate Links
//!
//! - [Documentation on docs.rs](https://docs.rs/egui_mobius_reactive)
//! - [Crate on crates.io](https://crates.io/crates/egui_mobius_reactive)
//! - [Source Code on GitHub](https://github.com/saturn77/egui_mobius)
//!
//! ## Notes
//!
//! This crate is lightweight and designed to empower developers to write concise, reactive code for complex UIs.
//! Contributions and issue reports are welcome—our GitHub repository is always open for community collaboration!

pub mod reactive;
pub use crate::reactive::{Dynamic, ValueExt, Derived, ReactiveList, ReactiveValue, SignalRegistry};

