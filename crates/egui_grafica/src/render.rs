//! egui `Painter` rendering of a [`crate::model::Scene`].
//!
//! This is the only module besides [`crate::citizen`] that depends on
//! egui. A future Masonry / wgpu / web backend would replace this
//! module wholesale without touching [`crate::model`] or [`crate::lang`].
//!
//! Not yet implemented — this is a placeholder. The intended surface:
//!
//! ```ignore
//! pub fn paint_scene(ui: &mut egui::Ui, scene: &Scene);
//! pub fn paint_node (painter: &egui::Painter, node: &Node, world_to_screen: &Transform2D);
//! pub fn paint_edge (painter: &egui::Painter, edge: &Edge, scene: &Scene);
//! ```

use crate::model::Scene;

/// Render a scene into the given egui `Ui`. Not yet implemented.
pub fn paint_scene(_ui: &mut egui::Ui, _scene: &Scene) {
    // TODO: walk scene.nodes, scene.edges; resolve port positions; paint shapes,
    // text, routed edges via egui::Painter primitives.
}
