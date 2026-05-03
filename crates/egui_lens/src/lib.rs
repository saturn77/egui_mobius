//! # Reactive Event Logger
//!
//! A reactive event logger for egui applications. This crate provides a customizable
//! terminal-like interface for logging events in egui applications. It's designed to
//! work with the egui_mobius_reactive crate for reactive state management.
//!
//! ## Features
//!
//! - Real-time logging in a terminal-like interface
//! - Customizable colors and visualization
//! - Support for different log levels (info, warning, error, debug)
//! - Flexible custom log types with string identifiers
//! - Configurable UI with column visibility options
//! - Export logs to file functionality
//! - Reactive architecture using egui_mobius_reactive

mod logger;
mod payload;
mod logger_colors;

pub use logger::{
    ReactiveEventLogger,
    ReactiveEventLoggerState,
    LogType,
};

pub use logger_colors::{LogColors, Color32Wrapper};
pub use payload::LoggerPayload;