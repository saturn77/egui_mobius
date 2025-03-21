//! Examples demonstrating various features of egui_mobius
//! 
//! This crate contains a collection of examples showing how to use different features
//! of the egui_mobius framework. Each example is a standalone binary that can be run
//! using `cargo run --example <example_name>`.
//! 
//! # Examples
//! 
//! ## Clock Async
//! ```bash
//! cargo run --example clock_async
//! ```
//! Demonstrates async operations with a clock widget. Shows how to:
//! - Use background threads for time updates
//! - Handle async operations in egui
//! - Implement thread-safe state management
//! 
//! ## Dashboard
//! ```bash
//! cargo run --example dashboard
//! ```
//! Shows how to create a dashboard with multiple widgets. Features:
//! - Multiple independent widgets
//! - State management across widgets
//! - Layout management
//! 
//! ## Dashboard Async
//! ```bash
//! cargo run --example dashboard_async
//! ```
//! Async version of the dashboard example, demonstrating:
//! - Background data fetching
//! - Async state updates
//! - Thread-safe widget communication
//! 
//! ## Reactive
//! ```bash
//! cargo run --example reactive
//! ```
//! Shows usage of the reactive system, including:
//! - `Value<T>` for state management
//! - Derived values with auto-updating
//! - Signal-slot connections
//! 
//! ## Realtime Plot
//! ```bash
//! cargo run --example realtime_plot
//! ```
//! Real-time plotting example showing:
//! - Integration with egui_plot
//! - Real-time data updates
//! - Smooth animations
//! 
//! ## Subscriber
//! ```bash
//! cargo run --example subscriber
//! ```
//! Demonstrates the subscriber pattern:
//! - Event subscription system
//! - Message passing between components
//! - Thread-safe event handling
//! 
//! ## UI Refresh Events
//! ```bash
//! cargo run --example ui_refresh_events
//! ```
//! Shows how to handle UI refresh events:
//! - Custom refresh triggers
//! - UI update scheduling
//! - State synchronization
