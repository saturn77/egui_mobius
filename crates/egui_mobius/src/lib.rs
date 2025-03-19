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
//! - **`Value<T>` State Management**: Thread-safe state container with proper
//!   synchronization
//! - **Background Operations**: Clean thread separation for non-blocking UI
//! - **Flexible Dispatching**: Support for both synchronous and asynchronous
//!   message dispatching with automatic thread management
//!
//! # Basic Example
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
//! # Advanced Dispatching
//!
//! The framework provides two main dispatching mechanisms:
//!
//! ## Synchronous Dispatcher
//!
//! ```rust
//! use egui_mobius::dispatching::{Dispatcher, SignalDispatcher};
//!
//! // Create a dispatcher for handling different message types
//! let mut dispatcher = Dispatcher::<String>::new();
//!
//! // Register a handler for processing messages
//! dispatcher.register_slot("channel1", |message| {
//!     println!("Received: {}", message);
//! });
//!
//! // Send messages through the dispatcher
//! dispatcher.send("channel1", "Hello, World!".to_string());
//! ```
//!
//! ## Asynchronous Dispatcher
//!
//! ```rust
//! use egui_mobius::dispatching::{AsyncDispatcher, SignalDispatcher};
//! use egui_mobius::factory::create_signal_slot;
//! use std::time::Duration;
//!
//! // Create an async dispatcher
//! let mut dispatcher = AsyncDispatcher::new();
//!
//! // Set up signal/slot for async processing
//! let (signal, slot) = create_signal_slot::<String>();
//! let (result_signal, result_slot) = create_signal_slot::<String>();
//!
//! // Attach async handler with timeout
//! dispatcher.attach_async(slot, result_signal, |input| async move {
//!     tokio::time::sleep(Duration::from_secs(1)).await;
//!     format!("Processed: {}", input)
//! });
//! ```
//!
//! The AsyncDispatcher is particularly useful for:
//! - Long-running background tasks
//! - Operations that shouldn't block the UI thread
//! - Parallel processing of multiple messages
//! - Handling timeouts and cancellation
//!
//! # Modules
//!
//! - [`signals`]: Signal type for sending messages
//! - [`slot`]: Slot type for receiving and processing messages
//! - [`factory`]: Utilities for creating signal-slot pairs
//! - [`types`]: Core types like `Value<T>` for state management
//! - [`dispatching`]: Signal dispatching and routing system, including AsyncDispatcher
//!

pub mod signals;
pub mod slot;
pub mod factory;
pub mod types; 
pub mod dispatching;
