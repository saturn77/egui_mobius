//! Connection routing.
//!
//! The router is the single source of routing truth: it turns an edge's
//! endpoint ports plus its routing strategy into a world-space polyline.
//! The renderer draws that polyline; the interaction layer hit-tests it.
//! Neither one re-derives the path.
//!
//! Working in world space (not screen space) keeps routing independent of
//! zoom/pan and lets the same polyline serve both rendering and hit-testing.

use crate::geometry;
use crate::model::{Edge, EdgeEnd, Node, NodeId, Port, PortAnchor, PortId, Routing, Scene};

/// Certified maximum distance, in world units, between a flattened bezier
/// polyline and the true curve. hypercurve guarantees the emitted polyline
/// stays within this bound.
const BEZIER_FLATNESS: f32 = 0.5;
/// Segment count for the fixed-step bezier fallback (used only if certified
/// flattening is somehow uncertain — it should not be, for a plain cubic).
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

/// World-space position of an edge end — the port's position, or the
/// free point itself. `None` if a port end references a missing port.
pub fn edge_end_position(scene: &Scene, end: &EdgeEnd) -> Option<(f32, f32)> {
    match end {
        EdgeEnd::Port(n, p) => port_world_position(scene, n, p),
        EdgeEnd::Free(x, y) => Some((*x, *y)),
    }
}

/// The routed path of an edge as a world-space polyline. `None` if a
/// port endpoint references a missing port.
pub fn edge_polyline(scene: &Scene, edge: &Edge) -> Option<Vec<(f32, f32)>> {
    let from = edge_end_position(scene, &edge.from)?;
    let to = edge_end_position(scene, &edge.to)?;
    Some(route(from, to, &edge.routing))
}

/// Route between two world points under a routing strategy.
pub fn route(from: (f32, f32), to: (f32, f32), routing: &Routing) -> Vec<(f32, f32)> {
    match routing {
        Routing::Straight => vec![from, to],
        Routing::Orthogonal => orthogonal(from, to),
        Routing::Manual { waypoints } => {
            let mut pts = Vec::with_capacity(waypoints.len() + 2);
            pts.push(from);
            pts.extend(waypoints.iter().copied());
            pts.push(to);
            pts
        }
        Routing::Bezier => bezier(from, to),
    }
}

/// Three-segment H-V-H orthogonal route through the horizontal midpoint.
fn orthogonal(from: (f32, f32), to: (f32, f32)) -> Vec<(f32, f32)> {
    let mid_x = (from.0 + to.0) * 0.5;
    vec![from, (mid_x, from.1), (mid_x, to.1), to]
}

/// Horizontal-out, horizontal-in cubic bezier between the endpoints,
/// flattened to a polyline. The control-handle length scales with horizontal
/// distance so short edges curve gently and long edges flow.
///
/// Flattening goes through hypercurve's certified flattener: the polyline is
/// provably within [`BEZIER_FLATNESS`] world units of the true curve, rather
/// than sampled at an arbitrary fixed step. The fixed-step [`sampled_cubic`]
/// is a fallback for the (not expected) case where the kernel reports the
/// flattening uncertain.
fn bezier(from: (f32, f32), to: (f32, f32)) -> Vec<(f32, f32)> {
    let handle = (to.0 - from.0).abs().max(40.0) * 0.5;
    let c1 = (from.0 + handle, from.1);
    let c2 = (to.0 - handle, to.1);
    flatten_cubic(from, c1, c2, to).unwrap_or_else(|| sampled_cubic(from, c1, c2, to))
}

/// Flatten a cubic bezier through hypercurve, with a certified flatness
/// bound. `None` if the kernel cannot certify the result.
fn flatten_cubic(
    p0: (f32, f32),
    c1: (f32, f32),
    c2: (f32, f32),
    p3: (f32, f32),
) -> Option<Vec<(f32, f32)>> {
    use hypercurve::{BezierFlatteningOptions, Classification, CubicBezier2, CurvePolicy};
    let policy = CurvePolicy::default();
    let options = BezierFlatteningOptions::try_new(geometry::real(BEZIER_FLATNESS), 16, &policy).ok()?;
    let curve = CubicBezier2::new(
        geometry::point(p0),
        geometry::point(c1),
        geometry::point(c2),
        geometry::point(p3),
    );
    match curve.flatten_certified(&options, &policy) {
        Classification::Decided(poly) => {
            Some(poly.points().iter().map(geometry::point_xy).collect())
        }
        Classification::Uncertain(_) => None,
    }
}

/// Fixed-step cubic bezier sampling — the fallback when certified flattening
/// reports uncertainty.
fn sampled_cubic(
    p0: (f32, f32),
    c1: (f32, f32),
    c2: (f32, f32),
    p3: (f32, f32),
) -> Vec<(f32, f32)> {
    (0..=BEZIER_SEGMENTS)
        .map(|i| cubic(p0, c1, c2, p3, i as f32 / BEZIER_SEGMENTS as f32))
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
    fn manual_route_threads_through_its_waypoints() {
        let p = route(
            (0.0, 0.0),
            (100.0, 40.0),
            &Routing::Manual { waypoints: vec![(80.0, 0.0), (80.0, 40.0)] },
        );
        assert_eq!(p, vec![(0.0, 0.0), (80.0, 0.0), (80.0, 40.0), (100.0, 40.0)]);
    }

    #[test]
    fn bezier_route_starts_and_ends_on_the_endpoints() {
        // Certified flattening keeps the curve endpoints exactly; the point
        // count depends on the subdivision the flatness bound requires.
        let p = route((0.0, 0.0), (100.0, 40.0), &Routing::Bezier);
        assert_eq!(p.first(), Some(&(0.0, 0.0)));
        assert_eq!(p.last(), Some(&(100.0, 40.0)));
        assert!(p.len() >= 2);
    }
}
