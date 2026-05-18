//! Interaction layer: hit-testing, selection, and node dragging.
//!
//! This module is renderer-aware — it works in world coordinates and the
//! caller maps screen points through a [`Viewport`](crate::render::Viewport)
//! first — but it does not own egui input handling. [`crate::citizen`] drives
//! it each frame: it reads pointer state from egui, calls into the helpers
//! here, and applies the resulting edits through the [`Registry`].
//!
//! [`Registry`]: crate::registry::Registry

use crate::model::{EdgeId, Node, NodeId, PortId, Scene};
use crate::render::{edge_world_polyline, port_position_on_node};

// =============================================================================
// Selection
// =============================================================================

/// The set of currently-selected scene elements — nodes and/or edges.
#[derive(Debug, Default, Clone)]
pub struct Selection {
    pub nodes: Vec<NodeId>,
    pub edges: Vec<EdgeId>,
}

impl Selection {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty() && self.edges.is_empty()
    }

    pub fn contains(&self, id: &NodeId) -> bool {
        self.nodes.iter().any(|n| n == id)
    }

    pub fn contains_edge(&self, id: &EdgeId) -> bool {
        self.edges.iter().any(|e| e == id)
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }

    /// Replace the whole selection with exactly one node.
    pub fn select_only(&mut self, id: NodeId) {
        self.clear();
        self.nodes.push(id);
    }

    /// Replace the whole selection with exactly one edge.
    pub fn select_only_edge(&mut self, id: EdgeId) {
        self.clear();
        self.edges.push(id);
    }

    /// Add the node if absent, remove it if present — shift-click behavior.
    pub fn toggle(&mut self, id: NodeId) {
        if let Some(pos) = self.nodes.iter().position(|n| n == &id) {
            self.nodes.remove(pos);
        } else {
            self.nodes.push(id);
        }
    }

    /// Add the edge if absent, remove it if present — shift-click behavior.
    pub fn toggle_edge(&mut self, id: EdgeId) {
        if let Some(pos) = self.edges.iter().position(|e| e == &id) {
            self.edges.remove(pos);
        } else {
            self.edges.push(id);
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
    /// Drawing a new connection from a source port. `cursor_world` tracks the
    /// pointer so the rubber-band preview can be drawn; on release, the
    /// nearest port to `cursor_world` becomes the edge's destination.
    Connecting {
        from: (NodeId, PortId),
        cursor_world: (f32, f32),
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

/// The port nearest to `world`, if one is within `radius` world units.
/// Used to start a connection (press near a port) and to finish one
/// (release near a port).
pub fn hit_test_port(scene: &Scene, world: (f32, f32), radius: f32) -> Option<(NodeId, PortId)> {
    let r2 = radius * radius;
    let mut best: Option<(f32, (NodeId, PortId))> = None;
    for node in &scene.nodes {
        for port in &node.ports {
            let (px, py) = port_position_on_node(node, port);
            let d2 = (px - world.0).powi(2) + (py - world.1).powi(2);
            if d2 <= r2 && best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
                best = Some((d2, (node.id.clone(), port.id.clone())));
            }
        }
    }
    best.map(|(_, ids)| ids)
}

// =============================================================================
// Snapping
// =============================================================================

/// The edge whose routed path passes nearest `world`, if one is within
/// `threshold` world units. Tests distance to the edge's polyline.
pub fn hit_test_edge(scene: &Scene, world: (f32, f32), threshold: f32) -> Option<EdgeId> {
    let mut best: Option<(f32, EdgeId)> = None;
    for edge in &scene.edges {
        let Some(poly) = edge_world_polyline(scene, edge) else {
            continue;
        };
        let d = polyline_distance(&poly, world);
        if d <= threshold && best.as_ref().is_none_or(|(bd, _)| d < *bd) {
            best = Some((d, edge.id.clone()));
        }
    }
    best.map(|(_, id)| id)
}

/// Shortest distance from `p` to a polyline (min over its segments).
fn polyline_distance(poly: &[(f32, f32)], p: (f32, f32)) -> f32 {
    poly.windows(2)
        .map(|seg| point_segment_distance(p, seg[0], seg[1]))
        .fold(f32::INFINITY, f32::min)
}

/// Distance from point `p` to the line segment `a`–`b`.
fn point_segment_distance(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let len2 = abx * abx + aby * aby;
    let t = if len2 <= f32::EPSILON {
        0.0
    } else {
        (((p.0 - a.0) * abx + (p.1 - a.1) * aby) / len2).clamp(0.0, 1.0)
    };
    let (cx, cy) = (a.0 + t * abx, a.1 + t * aby);
    ((p.0 - cx).powi(2) + (p.1 - cy).powi(2)).sqrt()
}

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
    fn hit_test_port_finds_nearest_within_radius() {
        use crate::model::{Port, PortAnchor, PortKind};
        let mut node = rect_node("n", (0.0, 0.0), (100.0, 100.0));
        // East(0.5) → world (100, 50).
        node.ports.push(Port {
            id: PortId("p".into()),
            name: "p".into(),
            kind: PortKind::Out,
            anchor: PortAnchor::East(0.5),
            data_type: None,
        });
        let mut scene = Scene::default();
        scene.nodes.push(node);

        assert_eq!(
            hit_test_port(&scene, (102.0, 51.0), 5.0),
            Some((NodeId("n".into()), PortId("p".into()))),
        );
        assert_eq!(hit_test_port(&scene, (130.0, 50.0), 5.0), None);
    }

    #[test]
    fn hit_test_edge_finds_a_wire_near_the_pointer() {
        use crate::model::{
            Edge, EdgeId, EdgeOverlay, Port, PortAnchor, PortId, PortKind, Routing,
        };
        let mut a = rect_node("a", (0.0, 0.0), (100.0, 100.0));
        a.ports.push(Port {
            id: PortId("pa".into()),
            name: "pa".into(),
            kind: PortKind::Out,
            anchor: PortAnchor::East(0.5), // world (100, 50)
            data_type: None,
        });
        let mut b = rect_node("b", (200.0, 0.0), (100.0, 100.0));
        b.ports.push(Port {
            id: PortId("pb".into()),
            name: "pb".into(),
            kind: PortKind::In,
            anchor: PortAnchor::West(0.5), // world (200, 50)
            data_type: None,
        });
        let mut scene = Scene::default();
        scene.nodes.push(a);
        scene.nodes.push(b);
        scene.edges.push(Edge {
            id: EdgeId("e".into()),
            from: (NodeId("a".into()), PortId("pa".into())),
            to: (NodeId("b".into()), PortId("pb".into())),
            routing: Routing::Straight,
            overlay: EdgeOverlay::default(),
        });

        assert_eq!(hit_test_edge(&scene, (150.0, 52.0), 5.0), Some(EdgeId("e".into())));
        assert_eq!(hit_test_edge(&scene, (150.0, 100.0), 5.0), None);
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
