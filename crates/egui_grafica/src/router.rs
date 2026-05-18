//! Connection routing.
//!
//! The router is the single source of routing truth: it turns an edge's
//! endpoint ports plus its routing strategy into a world-space polyline.
//! The renderer draws that polyline; the interaction layer hit-tests it.
//! Neither one re-derives the path.
//!
//! Working in world space (not screen space) keeps routing independent of
//! zoom/pan and lets the same polyline serve both rendering and hit-testing.

use crate::model::{Edge, Node, NodeId, Port, PortAnchor, PortId, Routing, Scene};

/// Number of straight segments a bezier route is sampled into. Enough to look
/// smooth and to give per-segment dash/dotted styling something to chew on.
const BEZIER_SEGMENTS: usize = 32;

// =============================================================================
// Port geometry
// =============================================================================

/// World-space position of a port, from its node's transform and the port's
/// parametric anchor.
pub fn port_position_on_node(node: &Node, port: &Port) -> (f32, f32) {
    let (x, y) = node.transform.position;
    let (w, h) = node.transform.size;
    match port.anchor {
        PortAnchor::North(t) => (x + w * t, y),
        PortAnchor::South(t) => (x + w * t, y + h),
        PortAnchor::East(t) => (x + w, y + h * t),
        PortAnchor::West(t) => (x, y + h * t),
        PortAnchor::Free(fx, fy) => (x + w * fx, y + h * fy),
    }
}

/// World-space position of a port given its node + port ids. Returns `None`
/// if either id is unknown — useful for skipping orphaned edges mid-edit.
pub fn port_world_position(scene: &Scene, node_id: &NodeId, port_id: &PortId) -> Option<(f32, f32)> {
    let node = scene.nodes.iter().find(|n| &n.id == node_id)?;
    let port = node.ports.iter().find(|p| &p.id == port_id)?;
    Some(port_position_on_node(node, port))
}

// =============================================================================
// Routing
// =============================================================================

/// The routed path of an edge as a world-space polyline. `None` if either
/// endpoint port is missing.
pub fn edge_polyline(scene: &Scene, edge: &Edge) -> Option<Vec<(f32, f32)>> {
    let from = port_world_position(scene, &edge.from.0, &edge.from.1)?;
    let to = port_world_position(scene, &edge.to.0, &edge.to.1)?;
    Some(route(from, to, &edge.routing))
}

/// Route between two world points under a routing strategy.
pub fn route(from: (f32, f32), to: (f32, f32), routing: &Routing) -> Vec<(f32, f32)> {
    match routing {
        Routing::Straight => vec![from, to],
        Routing::Orthogonal => orthogonal(from, to, 0.0),
        Routing::Manual { mid_offset } => orthogonal(from, to, *mid_offset),
        Routing::Bezier => bezier(from, to),
    }
}

/// Three-segment H-V-H orthogonal route. The vertical segment sits at the
/// horizontal midpoint plus `mid_offset` — zero for an automatic route, a
/// dragged value for a hand-routed one.
fn orthogonal(from: (f32, f32), to: (f32, f32), mid_offset: f32) -> Vec<(f32, f32)> {
    let mid_x = (from.0 + to.0) * 0.5 + mid_offset;
    vec![from, (mid_x, from.1), (mid_x, to.1), to]
}

/// Horizontal-out, horizontal-in cubic bezier, sampled to a polyline. The
/// control-handle length scales with horizontal distance so short edges
/// curve gently and long edges flow.
fn bezier(from: (f32, f32), to: (f32, f32)) -> Vec<(f32, f32)> {
    let handle = (to.0 - from.0).abs().max(40.0) * 0.5;
    let p1 = (from.0 + handle, from.1);
    let p2 = (to.0 - handle, to.1);
    (0..=BEZIER_SEGMENTS)
        .map(|i| cubic(from, p1, p2, to, i as f32 / BEZIER_SEGMENTS as f32))
        .collect()
}

fn cubic(p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32), t: f32) -> (f32, f32) {
    let u = 1.0 - t;
    let (b0, b1, b2, b3) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
    (
        b0 * p0.0 + b1 * p1.0 + b2 * p2.0 + b3 * p3.0,
        b0 * p0.1 + b1 * p1.1 + b2 * p2.1 + b3 * p3.1,
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn straight_route_is_two_points() {
        assert_eq!(route((0.0, 0.0), (10.0, 5.0), &Routing::Straight), vec![(0.0, 0.0), (10.0, 5.0)]);
    }

    #[test]
    fn orthogonal_route_turns_at_the_midpoint() {
        let p = route((0.0, 0.0), (100.0, 40.0), &Routing::Orthogonal);
        assert_eq!(p, vec![(0.0, 0.0), (50.0, 0.0), (50.0, 40.0), (100.0, 40.0)]);
    }

    #[test]
    fn manual_route_shifts_the_vertical_segment_by_the_offset() {
        let p = route((0.0, 0.0), (100.0, 40.0), &Routing::Manual { mid_offset: 30.0 });
        // Natural midpoint is x=50; offset 30 puts the vertical at x=80.
        assert_eq!(p, vec![(0.0, 0.0), (80.0, 0.0), (80.0, 40.0), (100.0, 40.0)]);
    }

    #[test]
    fn bezier_route_starts_and_ends_on_the_endpoints() {
        let p = route((0.0, 0.0), (100.0, 40.0), &Routing::Bezier);
        assert_eq!(p.first(), Some(&(0.0, 0.0)));
        assert_eq!(p.last(), Some(&(100.0, 40.0)));
        assert_eq!(p.len(), BEZIER_SEGMENTS + 1);
    }
}
