//! # egui_grafica
//!
//! Programmable graphics canvas citizen for egui_mobius applications.
//! Supports system block diagrams (draw.io style) and node graphs from
//! the same `Node + Port + Edge + Overlay` data model, authored in a
//! first-class `.canvas` domain-specific language.
//!
//! ## Module layout
//!
//! - [`model`] — pure data: `Scene`, `Node`, `Port`, `Edge`, `Overlay`.
//!   No egui dependency. This is the durable artifact.
//! - [`lang`] — the `.canvas` DSL: lexer, parser, AST, pretty-printer.
//!   The source file on disk is the authoritative artifact.
//! - [`render`] — egui `Painter` implementation of `Scene` →  pixels.
//! - [`interact`] — selection, drag, port-to-port connect, marquee,
//!   snap-to-grid.
//! - [`citizen`] — `CanvasCitizen`, the dock-panel integration that
//!   makes a `Scene` first-class within an `egui_mobius` application.
//!
//! Only [`render`] and [`citizen`] depend on egui. A future
//! `grafica-masonry` or `grafica-wgpu` backend replaces exactly those
//! two modules.

pub mod model;
pub mod lang;
pub mod render;
pub mod interact;
pub mod citizen;

pub use model::{
    Edge, EdgeId, EdgeOverlay, Fill, Border, Group, Node, NodeId, NodeKind,
    Overlay, Port, PortAnchor, PortId, PortKind, Routing, Scene,
    CanvasSettings, TextLabel, Transform,
};
