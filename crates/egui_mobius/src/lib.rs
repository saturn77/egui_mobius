//! egui_mobius - A thread-aware signal/slot framework for egui applications with reactive capabilities
//!
//! egui_mobius provides a robust signal/slot architecture inspired by Qt, designed
//! specifically for egui applications. It enables clean separation between UI and
//! background operations through type-safe message passing and thread management.
//!
//! # Key Features
//!
//! - **Thread-aware Slot System**: Each slot maintains its own thread for true
//!   hybrid sync/async operation
//! - **Type-safe Message Passing**: Strongly typed signals and slots ensure
//!   compile-time correctness
//! - **Signal-Slot Architecture**: Qt-inspired design pattern for decoupled
//!   communication
//! - **`Value<T>` State Management**: Thread-safe state container with proper
//!   synchronization
//! - **Background Operations**: Clean thread separation for non-blocking UI
//! - **Flexible Dispatching**: Support for both synchronous and asynchronous
//!   message dispatching with automatic thread management
//! - **Reactive System**: Value change notifications and auto-updating computed values through a signal registry
//!
//! # Examples
//!
//! ## Reactive System
//!
//! ```rust
//! use egui_mobius::types::Value;
//! use egui_mobius::reactive::Derived;
//! 
//! let count = Value::new(0);
//! let count_clone = count.clone();
//! let doubled = Derived::new(&[count.clone()], move || {
//!     let val = *count_clone.lock().unwrap();
//!     val * 2
//! });
//! 
//! assert_eq!(doubled.get(), 0);
//! *count.lock().unwrap() = 5;
//! std::thread::sleep(std::time::Duration::from_millis(200));
//! assert_eq!(doubled.get(), 10);
//! ```
//!
//! # Module Overview
//!
//! The framework consists of several key modules:
//!
//! - [`signals`]: Signal type for sending messages
//! - [`slot`]: Slot type for receiving and processing messages
//! - [`factory`]: Utilities for creating signal-slot pairs
//! - [`types`]: Core types like `Value<T>` for state management
//! - [`dispatching`]: Signal dispatching and routing system
//! - [`reactive`]: Reactive system with value change notifications and computed values

// Declare modules
pub mod signals;
pub mod slot;
pub mod types;
pub mod factory;
pub mod dispatching;
pub mod reactive;

// Re-export commonly used items
pub use signals::Signal;
pub use slot::Slot;
pub use types::Value;
pub use reactive::{ValueExt, SignalValue, Derived};
pub use dispatching::{Dispatcher, SignalDispatcher, AsyncDispatcher};
