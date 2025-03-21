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
//! ## Basic Signal-Slot Usage
//!
//! ```rust
//! use egui_mobius::factory::create_signal_slot;
//! use egui_mobius::types::Value;
//! use std::collections::VecDeque;
//!
//! // Define message types for type-safe communication
//! #[derive(Clone)]
//! enum UiEvent {
//!     AddTask { id: u32, description: String },
//!     CompleteTask { id: u32 },
//! }
//!
//! #[derive(Clone)]
//! enum BackendResponse {
//!     TaskAdded { id: u32 },
//!     TaskCompleted { id: u32 },
//! }
//!
//! // Application state
//! struct AppState {
//!     tasks: Value<VecDeque<String>>,
//!     needs_refresh: Value<bool>,
//!     ui_signal: Signal<UiEvent>,
//!     ui_slot: Slot<BackendResponse>,
//! }
//!
//! impl AppState {
//!     fn new(
//!         ui_signal: Signal<UiEvent>,
//!         mut ui_slot: Slot<BackendResponse>,
//!     ) -> Self {
//!         // Create shared state
//!         let tasks = Value::new(VecDeque::new());
//!         let needs_refresh = Value::new(false);
//!
//!         // Set up UI response handler
//!         let needs_refresh_clone = Value::clone(&needs_refresh);
//!         ui_slot.start(move |response| {
//!             *needs_refresh_clone.lock().unwrap() = true;
//!             match response {
//!                 BackendResponse::TaskAdded { id } => {
//!                     println!("UI: Task {} added", id);
//!                 }
//!                 BackendResponse::TaskCompleted { id } => {
//!                     println!("UI: Task {} completed", id);
//!                 }
//!             }
//!         });
//!
//!         Self {
//!             tasks,
//!             needs_refresh,
//!             ui_signal,
//!             ui_slot,
//!         }
//!     }
//!
//!     fn add_task(&self, description: String) {
//!         self.ui_signal.send(UiEvent::AddTask {
//!             id: 1,
//!             description,
//!         }).unwrap();
//!     }
//! }
//!
//! fn start_backend_thread(
//!     tasks: Value<VecDeque<String>>,
//!     needs_refresh: Value<bool>,
//!     mut backend_slot: Slot<UiEvent>,
//!     backend_signal: Signal<BackendResponse>,
//! ) {
//!     backend_slot.start(move |event| {
//!         let mut task_list = tasks.lock().unwrap();
//!         
//!         match event {
//!             UiEvent::AddTask { id, description } => {
//!                 task_list.push_back(description);
//!                 *needs_refresh.lock().unwrap() = true;
//!                 backend_signal.send(BackendResponse::TaskAdded { id }).unwrap();
//!             }
//!             UiEvent::CompleteTask { id } => {
//!                 if !task_list.is_empty() {
//!                     task_list.pop_front();
//!                     *needs_refresh.lock().unwrap() = true;
//!                     backend_signal.send(BackendResponse::TaskCompleted { id }).unwrap();
//!                 }
//!             }
//!         }
//!     });
//! }
//!
//! fn main() {
//!     // Create communication channels
//!     let (ui_signal, backend_slot) = create_signal_slot::<UiEvent>();
//!     let (backend_signal, ui_slot) = create_signal_slot::<BackendResponse>();
//!
//!     // Create application state
//!     let app = AppState::new(ui_signal.clone(), ui_slot);
//!
//!     // Start backend processing thread
//!     start_backend_thread(
//!         Value::clone(&app.tasks),
//!         Value::clone(&app.needs_refresh),
//!         backend_slot,
//!         backend_signal,
//!     );
//!
//!     // Use the application
//!     app.add_task("Learn egui_mobius".to_string());
//! }
//! ```
//!
//! ## Async Message Dispatching
//!
//! ```rust
//! use egui_mobius::dispatching::AsyncDispatcher;
//! use egui_mobius::factory::create_signal_slot;
//! use std::time::Duration;
//!
//! // Define message types
//! #[derive(Clone)]
//! enum Request {
//!     FetchWeather(String),  // city name
//!     FetchTime(String),     // timezone
//! }
//!
//! #[derive(Clone)]
//! enum Response {
//!     Weather { city: String, temp: f32 },
//!     Time { zone: String, time: String },
//! }
//!
//! // Create bidirectional channels
//! let (signal_to_dispatcher, slot_from_ui) = create_signal_slot::<Request>();
//! let (signal_to_ui, slot_from_dispatcher) = create_signal_slot::<Response>();
//!
//! // Create and configure the dispatcher
//! let dispatcher = AsyncDispatcher::<Request, Response>::new();
//! let signal_to_ui = signal_to_ui.clone();
//!
//! // Attach async handler
//! dispatcher.attach_async(slot_from_ui, signal_to_ui, |request| async move {
//!     match request {
//!         Request::FetchWeather(city) => {
//!             // Simulate API call
//!             tokio::time::sleep(Duration::from_millis(100)).await;
//!             Response::Weather {
//!                 city,
//!                 temp: 22.5,
//!             }
//!         }
//!         Request::FetchTime(zone) => {
//!             tokio::time::sleep(Duration::from_millis(50)).await;
//!             Response::Time {
//!                 zone,
//!                 time: "10:30 AM".to_string(),
//!             }
//!         }
//!     }
//! });
//!
//! // UI can send requests
//! signal_to_dispatcher.send(Request::FetchWeather("London".to_string())).unwrap();
//!
//! // UI can handle responses
//! slot_from_dispatcher.start(|response| {
//!     match response {
//!         Response::Weather { city, temp } => {
//!             println!("Temperature in {}: {}°C", city, temp);
//!         }
//!         Response::Time { zone, time } => {
//!             println!("Time in {}: {}", zone, time);
//!         }
//!     }
//! });
//! ```
//!
//! ## Reactive System
//!
//! The reactive system, including `SignalRegistry`, `ValueExt`, and `Derived` types,
//! is available in the separate `egui_mobius_reactive` crate. This includes:
//!
//! - Value change notifications through the `ValueExt` trait
//! - Auto-updating computed values with `Derived`
//! - Signal registry for managing reactive dependencies
//! - Thread-safe reactive state management
//!
//! See the `egui_mobius_reactive` crate documentation for comprehensive examples
//! and usage patterns.
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
//!
//! The reactive system functionality is available in the separate `egui_mobius_reactive` crate.

// Declare modules
pub mod signals;
pub mod slot;
pub mod types;
pub mod factory;
pub mod dispatching;

// Re-export commonly used items
pub use signals::Signal;
pub use slot::Slot;
pub use types::Value;
pub use dispatching::{Dispatcher, SignalDispatcher, AsyncDispatcher};
