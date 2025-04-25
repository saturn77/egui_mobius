//! Ergonomic re-exports for `egui_mobius_components`
//!
//! Bring this into scope via:
//! ```rust
//! use egui_mobius_components::prelude::*;
//! ```

// Re-export event logger prelude
pub use crate::components::event_logger::prelude::*;

// Useful shared types
pub use std::sync::{Arc, Mutex};