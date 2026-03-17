//! Ergonomic re-exports for `egui_mobius_reactive`
//!
//! Bring this into scope via:
//! ```rust
//! use egui_mobius_reactive::*;
//! ```

pub use super::{
    core::{ReactiveList, ReactiveValue, Subscribers},
    derived::Derived,
    dynamic::{Dynamic, ValueExt},
    reactive_math::{ReactiveListSum, ReactiveLogic, ReactiveMath, ReactiveString},
    reactive_state::ReactiveWidgetRef,
    registry::SignalRegistry,
};

#[cfg(feature = "widgets")]
pub use super::{
    // Widgets
    widgets::ReactiveSlider,
};

// Useful shared types
pub use std::sync::{Arc, Mutex};
