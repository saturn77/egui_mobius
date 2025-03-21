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
//! # Quick Start
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
//! // Update the original value and see automatic updates
//! *count.lock().unwrap() = 5;
//! assert_eq!(*doubled.get(), 10);
//! ```
//! 
//! # Architecture
//!
//! The reactive system is built on three key architectural components that work together
//! to provide thread-safe, real-time UI updates:
//!
//! ## Key Components
//!
//! 1. **Thread-Safe Values**:
//!    - `Value<T>` wraps state in `Arc<Mutex<T>>`
//!    - Safe concurrent access across UI and worker threads
//!    - Change notification through `ValueExt` trait
//!
//! 2. **Automatic Dependency Tracking**:
//!    - `Derived<T>` computes values from dependencies
//!    - Auto-updates when dependencies change
//!    - Thread-safe computation in background
//!
//! 3. **Signal Management**:
//!    - `SignalRegistry` manages reactive context
//!    - Handles signal-slot connections
//!    - Prevents memory leaks
//!    - Type-safe message passing
//!
//! ## Complete Example
//!
//! ```rust
//! use std::sync::Arc;
//! use egui_mobius_reactive::{Value, Derived, SignalRegistry};
//!
//! // Define your application state
//! pub struct AppState {
//!     pub registry: SignalRegistry,
//!     count: Value<i32>,
//!     label: Value<String>,
//!     doubled: Derived<i32>,
//! }
//!
//! impl AppState {
//!     pub fn new(registry: SignalRegistry) -> Self {
//!         let count = Value::new(0);
//!         
//!         // Create a derived value that auto-updates when count changes
//!         let count_ref = count.clone();
//!         let doubled = Derived::new(&[count_ref.clone()], move || {
//!             let val = *count_ref.lock();
//!             val * 2
//!         });
//!         
//!         // Create UI label
//!         let label = Value::new("Click to increment".to_string());
//!
//!         // Register with SignalRegistry for lifecycle management
//!         registry.register_signal(Arc::new(count.clone()));
//!         registry.register_signal(Arc::new(doubled.clone()));
//!         
//!         Self { 
//!             registry,
//!             count,
//!             label,
//!             doubled,
//!         }
//!     }
//!
//!     pub fn increment(&self) {
//!         let new_count = *self.count.lock() + 1;
//!         self.count.set(new_count);
//!         // Doubled value updates automatically!
//!     }
//! }
//! ```
//!
//! # Features & Best Practices
//!
//! ## Thread Safety & Performance
//!
//! - All values protected by `Arc<Mutex<T>>`
//! - Each slot runs in its own thread for true async operation
//! - Clean thread separation for background tasks
//! - Type-safe message passing between threads
//!
//! ## Automatic Updates
//!
//! - Derived values update when dependencies change
//! - No manual synchronization needed
//! - UI updates trigger reactive updates
//! - Seamless integration with egui
//!
//! ## Memory Management
//!
//! - SignalRegistry handles registration and cleanup
//! - Prevents memory leaks from orphaned signals
//! - Automatic cleanup of unused values
//! - Manual cleanup available when needed
//!
//! ## Best Practices
//!
//! 1. **State Organization**:
//!    - Keep SignalRegistry at app level
//!    - Group related values in structs
//!    - Register all dependent values
//!
//! 2. **Thread Safety**:
//!    - Use Value::lock() for access
//!    - Clone before moving to closures
//!    - Let each slot handle its thread
//!


pub mod reactive;

// Re-export commonly used types for convenience
pub use reactive::{Value, Derived, SignalRegistry};
