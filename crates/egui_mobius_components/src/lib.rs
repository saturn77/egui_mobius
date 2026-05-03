//! # egui_mobius_components
//!
//! **DEPRECATED — superseded by [`egui_lens`](https://crates.io/crates/egui_lens).**
//!
//! This crate is the predecessor event logger built on the older
//! `egui_mobius` signal/slot architecture. New code should depend on
//! `egui_lens` directly, which provides the same logger functionality
//! using `Dynamic<T>` reactive primitives. This crate is frozen at
//! 0.4.0 and will receive no further updates.
//!
//! Migration: see `examples/logger_component` in the egui_mobius
//! workspace for a port of the lens-equivalent example.

#![deprecated(
    since = "0.4.0",
    note = "use `egui_lens` instead — egui_mobius_components is the predecessor logger superseded by egui_lens"
)]

pub mod components;
pub mod prelude;

// Re-export prelude for convenience
pub use prelude::*;
