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
pub mod geometry;
pub mod router;
pub mod render;
pub mod interact;
pub mod citizen;
pub mod icons;
pub mod export;
pub mod inspector;
pub mod page;
pub mod registry;

/// Retained wgpu rendering pipeline. Present only with the `gpu` feature;
/// see `develop/gpu_rendering_plan.md`.
#[cfg(feature = "gpu")]
pub mod gpu;

pub use model::{
    Edge, EdgeEnd, EdgeEndSide, EdgeId, EdgeOverlay, Fill, Border, Group, Node, NodeId, NodeKind,
    Overlay, Port, PortAnchor, PortId, PortKind, Routing, Scene,
    CanvasBackground, CanvasSettings, GridStyle, GridUnits, TextLabel, Transform,
};
pub use registry::Registry;
pub use citizen::{CanvasCitizen, RibbonSide, ShapeTool};
pub use render::Viewport;
pub use interact::{hit_test_node, snap_to_grid, Selection};

/// No-op kept for API compatibility.
///
/// The ribbon used to need the Phosphor icon font installed here; it now draws
/// its glyphs from Unicode/emoji in egui's *default* fonts (see [`icons`]), so
/// there is nothing to install. Existing callers can keep calling this or drop
/// the call entirely.
///
/// ```ignore
/// Box::new(|cc| {
///     egui_grafica::install_fonts(&cc.egui_ctx); // optional now
///     Ok(Box::new(MyApp::new(cc)))
/// });
/// ```
pub fn install_fonts(_ctx: &egui::Context) {}
