//! # egui_mobius_components
//!
//! A collection of reusable UI components for the egui_mobius framework.
//! Components use the egui_mobius signal/slot architecture for reactive UI.
//!
//! ## Components
//!
//! - Event Logger: A terminal-like widget for logging events
//!
//! ## Usage
//!
//! Components can be used directly or via the prelude:
//!
//! ```rust
//! use egui_mobius_components::prelude::*;
//! ```

pub mod components;
pub mod prelude;

// Re-export prelude for convenience
pub use prelude::*;