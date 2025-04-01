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
//! Here's how you can use `Dynamic` and `Derived` to build reactive state
//! management in your application. Note the use of the SignalRegistry struct
//! to keep the values from being dropped later in the function.
//!
//! ```rust
//! use egui_mobius_reactive::{Dynamic, Derived, SignalRegistry};
//! use std::sync::Arc;
//!
//!
//!  let registry = SignalRegistry::new();
//!
//!  let count = Dynamic::new(0);
//!  let count_for_compute = count.clone();
//!  let doubled = Derived::new(&[Arc::new(count.clone())], move || {
//!     *count_for_compute.lock() * 2
//!  });
//!
//!  // Register the values
//!  registry.register_named_signal("count", Arc::new(count.clone()));
//!  registry.register_named_signal("doubled", Arc::new(doubled.clone()));
//!
//!  // Values should still work after registration
//!  assert_eq!(doubled.get(), 0);
//!  count.set(5);
//!  std::thread::sleep(std::time::Duration::from_millis(10));
//!  assert_eq!(doubled.get(), 10);
//!
//! ```
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
pub use reactive::prelude::*;

// pub mod reactive;

// pub mod prelude {
//     pub use crate::reactive::prelude::*;
// }
