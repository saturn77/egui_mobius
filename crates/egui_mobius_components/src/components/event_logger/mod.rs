//! Event Logger Component for egui_mobius_components
//!
//! This module provides a reactive terminal-like widget for logging events
//! in Egui applications. It uses the egui_mobius signal/slot architecture
//! for asynchronous event handling.

// Core modules
pub mod log_colors;
pub mod log_type;
pub mod logger;
pub mod logger_state;
pub mod processor;
pub mod messages;
pub mod serialization;
pub mod platform;
pub mod prelude;