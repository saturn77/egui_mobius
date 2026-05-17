//! Interaction layer: hit-testing, selection, and node dragging.
//!
//! This module is renderer-aware — it works in world coordinates and the
//! caller maps screen points through a [`Viewport`](crate::render::Viewport)
//! first — but it does not own egui input handling. [`crate::citizen`] drives
//! it each frame: it reads pointer state from egui, calls into the helpers
//! here, and applies the resulting edits through the [`Registry`].
//!
//! [`Registry`]: crate::registry::Registry

use crate::model::{Node, NodeId, Scene};

// =============================================================================
// Selection
// =============================================================================

/// The set of currently-selected scene elements.
///
/// Node-only for now; edge selection lands when edge editing does.
#[derive(Debug, Default, Clone)]
pub struct Selection {
    pub nodes: Vec<NodeId>,
}

impl Selection {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn contains(&self, id: &NodeId) -> bool {
        self.nodes.iter().any(|n| n == id)
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
    }

    /// Replace the selection with exactly one node.
    pub fn select_only(&mut self, id: NodeId) {
        self.nodes.clear();
        self.nodes.push(id);
    }

    /// Add the node if absent, remove it if present — shift-click behavior.
    pub fn toggle(&mut self, id: NodeId) {
        if let Some(pos) = self.nodes.iter().position(|n| n == &id) {
            self.nodes.remove(pos);
        } else {
            self.nodes.push(id);
        }
    }
}

// =============================================================================
// Drag state
// =============================================================================

/// The in-progress pointer gesture on the canvas.
#[derive(Debug, Clone, Default)]
pub enum DragState {
    /// No drag in progress.
    #[default]
    Idle,
    /// Panning the viewport.
    Pan,
    /// Moving selected nodes.
    ///
    /// `origins` records each dragged node's position at the moment the drag
    /// began. Moves are applied as absolute `origin + delta` offsets rather
    /// than accumulated per frame — accumulation drifts under snapping and
    /// rounding.
    Nodes {
        grab_world: (f32, f32),
        origins: Vec<(NodeId, (f32, f32))>,
    },
}

// =============================================================================
// Hit testing
// =============================================================================

/// Topmost (last-drawn) node whose axis-aligned bounds contain `world`.
///
/// Bounding-box test for all node kinds — exact circle/ellipse hit testing is
/// a refinement that can come later; the bounding box is predictable and good
/// enough for selection and drag.
pub fn hit_test_node(scene: &Scene, world: (f32, f32)) -> Option<NodeId> {
    scene
        .nodes
        .iter()
        .rev()
        .find(|n| node_bounds_contains(n, world))
        .map(|n| n.id.clone())
}

fn node_bounds_contains(node: &Node, world: (f32, f32)) -> bool {
    let (x, y) = node.transform.position;
    let (w, h) = node.transform.size;
    world.0 >= x && world.0 <= x + w && world.1 >= y && world.1 <= y + h
}

// =============================================================================
// Snapping
// =============================================================================

/// Snap a world coordinate to the nearest grid multiple. Returns `pos`
/// unchanged when `spacing` is non-positive.
pub fn snap_to_grid(pos: (f32, f32), spacing: f32) -> (f32, f32) {
    if spacing <= 0.0 {
        return pos;
    }
    (
        (pos.0 / spacing).round() * spacing,
        (pos.1 / spacing).round() * spacing,
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NodeKind, Overlay, Transform};

    fn rect_node(id: &str, pos: (f32, f32), size: (f32, f32)) -> Node {
        Node {
            id: NodeId(id.to_string()),
            kind: NodeKind::Rect,
            transform: Transform { position: pos, size, rotation: 0.0 },
            overlay: Overlay::default(),
            ports: vec![],
        }
    }

    #[test]
    fn hit_test_picks_topmost() {
        let mut scene = Scene::default();
        scene.nodes.push(rect_node("under", (0.0, 0.0), (100.0, 100.0)));
        scene.nodes.push(rect_node("over", (10.0, 10.0), (50.0, 50.0)));
        // Point inside both — later-drawn "over" wins.
        assert_eq!(hit_test_node(&scene, (30.0, 30.0)), Some(NodeId("over".into())));
        // Point inside only the lower node.
        assert_eq!(hit_test_node(&scene, (90.0, 90.0)), Some(NodeId("under".into())));
        // Point outside both.
        assert_eq!(hit_test_node(&scene, (500.0, 500.0)), None);
    }

    #[test]
    fn selection_toggle_and_select_only() {
        let mut sel = Selection::default();
        sel.toggle(NodeId("a".into()));
        sel.toggle(NodeId("b".into()));
        assert!(sel.contains(&NodeId("a".into())));
        assert!(sel.contains(&NodeId("b".into())));
        sel.toggle(NodeId("a".into()));
        assert!(!sel.contains(&NodeId("a".into())));
        sel.select_only(NodeId("c".into()));
        assert_eq!(sel.nodes, vec![NodeId("c".into())]);
    }

    #[test]
    fn snap_rounds_to_nearest_multiple() {
        assert_eq!(snap_to_grid((23.0, 47.0), 10.0), (20.0, 50.0));
        assert_eq!(snap_to_grid((23.0, 47.0), 0.0), (23.0, 47.0));
    }
}
