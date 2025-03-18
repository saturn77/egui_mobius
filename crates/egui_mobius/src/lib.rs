//! egui_mobius - A thread-aware signal/slot framework for egui applications
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
//! - **Value<T> State Management**: Thread-safe state container with proper
//!   synchronization
//! - **Background Operations**: Clean thread separation for non-blocking UI
//!
//! # Example
//!
//! ```rust
//! use egui_mobius::factory::create_signal_slot;
//! use egui_mobius::signals::Signal;
//! use egui_mobius::slot::Slot;
//!
//! // Create a signal-slot pair for string messages
//! let (signal, mut slot) = create_signal_slot::<String>();
//!
//! // Set up a handler for the slot
//! slot.start(|message| {
//!     println!("Received: {}", message);
//! });
//!
//! // Send a message through the signal
//! signal.send("Hello from egui_mobius!".to_string()).unwrap();
//! ```
//!
//! # Modules
//!
//! - [`signals`]: Signal type for sending messages
//! - [`slot`]: Slot type for receiving and processing messages
//! - [`factory`]: Utilities for creating signal-slot pairs
//! - [`types`]: Core types like Value<T> for state management
//! - [`dispatching`]: Signal dispatching and routing system
//!

pub mod signals;
pub mod slot;
pub mod factory;
pub mod types; 
pub mod dispatching;
