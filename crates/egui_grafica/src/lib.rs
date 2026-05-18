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
pub mod router;
pub mod render;
pub mod interact;
pub mod citizen;
pub mod registry;

pub use model::{
    Edge, EdgeId, EdgeOverlay, Fill, Border, Group, Node, NodeId, NodeKind,
    Overlay, Port, PortAnchor, PortId, PortKind, Routing, Scene,
    CanvasSettings, GridStyle, GridUnits, TextLabel, Transform,
};
pub use registry::Registry;
pub use citizen::CanvasCitizen;
pub use render::Viewport;
pub use interact::{hit_test_node, snap_to_grid, Selection};

/// Install the Phosphor icon font into an egui context. Call once at app
/// startup — typically from `eframe::CreationContext` — so ribbon icons
/// render rather than appearing as tofu boxes.
///
/// ```ignore
/// Box::new(|cc| {
///     egui_grafica::install_fonts(&cc.egui_ctx);
///     Ok(Box::new(MyApp::new(cc)))
/// });
/// ```
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}
