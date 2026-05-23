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

/// The anchor of a port end — `None` for `EdgeEnd::Free` or for a port
/// that has been deleted.
pub fn edge_end_anchor(scene: &Scene, end: &EdgeEnd) -> Option<PortAnchor> {
    match end {
        EdgeEnd::Port(n, p) => scene
            .nodes
            .iter()
            .find(|nd| &nd.id == n)
            .and_then(|nd| nd.ports.iter().find(|pt| &pt.id == p))
            .map(|pt| pt.anchor),
        EdgeEnd::Free(..) => None,
    }
}

/// The routed path of an edge as a world-space polyline. `None` if a
/// port endpoint references a missing port.
///
/// Orthogonal routes use the port-direction-aware variant when both
/// endpoints are ports: wires exit perpendicular to each port's face
/// (N/S → vertical, E/W → horizontal), so they don't run co-linear
/// with the shape's side. Free endpoints fall back to the mid-X
/// H-V-H route since there's no port face to be perpendicular to.
pub fn edge_polyline(scene: &Scene, edge: &Edge) -> Option<Vec<(f32, f32)>> {
    let from = edge_end_position(scene, &edge.from)?;
    let to = edge_end_position(scene, &edge.to)?;
    match &edge.routing {
        Routing::Orthogonal => {
            let from_anchor = edge_end_anchor(scene, &edge.from);
            let to_anchor = edge_end_anchor(scene, &edge.to);
            Some(orthogonal_with_anchors(from, to, from_anchor, to_anchor))
        }
        Routing::Bezier => {
            let from_anchor = edge_end_anchor(scene, &edge.from);
            let to_anchor = edge_end_anchor(scene, &edge.to);
            Some(bezier_with_anchors(from, to, from_anchor, to_anchor))
        }
        other => Some(route(from, to, other)),
    }
}

/// Route between two world points under a routing strategy, with no
/// knowledge of port direction. Callers that *do* have anchors should
/// reach [`edge_polyline`] instead.
pub fn route(from: (f32, f32), to: (f32, f32), routing: &Routing) -> Vec<(f32, f32)> {
    match routing {
        Routing::Straight => vec![from, to],
        Routing::Orthogonal => orthogonal_mid_x(from, to),
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

/// Length of the stub segment emitted perpendicular to each port's
/// face before the route turns. World units. Sized so a stub is clearly
/// visible even for short routes without dominating long ones.
const PORT_STUB: f32 = 20.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ExitAxis {
    Horizontal, // East / West ports — wire exits along X
    Vertical,   // North / South ports — wire exits along Y
}

/// Unit-vector exit direction for a port anchor. `None` for
/// [`PortAnchor::Free`] — its placement doesn't define a face.
fn exit_dir(anchor: PortAnchor) -> Option<(f32, f32)> {
    match anchor {
        PortAnchor::North(_) => Some((0.0, -1.0)),
        PortAnchor::South(_) => Some((0.0, 1.0)),
        PortAnchor::East(_) => Some((1.0, 0.0)),
        PortAnchor::West(_) => Some((-1.0, 0.0)),
        PortAnchor::Free(..) => None,
    }
}

fn exit_axis(anchor: PortAnchor) -> Option<ExitAxis> {
    match anchor {
        PortAnchor::North(_) | PortAnchor::South(_) => Some(ExitAxis::Vertical),
        PortAnchor::East(_) | PortAnchor::West(_) => Some(ExitAxis::Horizontal),
        PortAnchor::Free(..) => None,
    }
}

/// Port-direction-aware orthogonal route. Both ends emit a stub
/// perpendicular to their port face; the stubs are joined by an
/// axis-aligned bend at a corner whose orientation matches the
/// from-end's exit axis. Falls back to [`orthogonal_mid_x`] when
/// either endpoint lacks a defined exit direction (Free / unknown).
fn orthogonal_with_anchors(
    from: (f32, f32),
    to: (f32, f32),
    from_anchor: Option<PortAnchor>,
    to_anchor: Option<PortAnchor>,
) -> Vec<(f32, f32)> {
    let (Some(fa), Some(ta)) = (from_anchor, to_anchor) else {
        return orthogonal_mid_x(from, to);
    };
    let (Some(f_dir), Some(t_dir)) = (exit_dir(fa), exit_dir(ta)) else {
        return orthogonal_mid_x(from, to);
    };
    let p1 = (from.0 + f_dir.0 * PORT_STUB, from.1 + f_dir.1 * PORT_STUB);
    let p2 = (to.0 + t_dir.0 * PORT_STUB, to.1 + t_dir.1 * PORT_STUB);

    // The corner that joins the two stubs sits where the from-stub's
    // axis meets the to-stub's axis — placement depends on whichever
    // axis the from-end exits along.
    let corner = match exit_axis(fa) {
        Some(ExitAxis::Horizontal) => (p1.0, p2.1),
        Some(ExitAxis::Vertical) => (p2.0, p1.1),
        None => return orthogonal_mid_x(from, to),
    };

    // Construct the polyline; dedup catches degenerate cases where
    // the corner coincides with one of the stub ends (aligned ports).
    let mut pts = vec![from, p1, corner, p2, to];
    pts.dedup();
    pts
}

/// Three-segment H-V-H orthogonal route through the horizontal midpoint.
/// The fallback used when port direction is unknown.
fn orthogonal_mid_x(from: (f32, f32), to: (f32, f32)) -> Vec<(f32, f32)> {
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

/// Port-direction-aware bezier route. The control handle at each end
/// is offset along that port's exit direction so the curve leaves
/// perpendicular to the port's face — a south-facing port enters from
/// below, an east-facing port exits to the right. The handle length
/// scales with the straight-line distance between endpoints, clamped
/// so short edges still curve and long edges don't swing wildly.
///
/// Falls back to a horizontal default when an endpoint is `Free` (no
/// face). Same flattening path as [`bezier`] — hypercurve's certified
/// flattener with the fixed-step sampler as fallback.
fn bezier_with_anchors(
    from: (f32, f32),
    to: (f32, f32),
    from_anchor: Option<PortAnchor>,
    to_anchor: Option<PortAnchor>,
) -> Vec<(f32, f32)> {
    let dx = to.0 - from.0;
    let dy = to.1 - from.1;
    let dist = (dx * dx + dy * dy).sqrt();
    // 40% of straight-line distance, clamped: short edges still curve
    // visibly, long edges don't fling control points off the canvas.
    let handle = (dist * 0.4).clamp(20.0, 200.0);

    // Default to horizontal-out / horizontal-in when an endpoint
    // has no face (Free dangle).
    let f_dir = from_anchor.and_then(exit_dir).unwrap_or((1.0, 0.0));
    // The to-side handle sits on the OUTSIDE of the to port — along
    // its exit direction — so the curve enters perpendicular to the
    // port's face.
    let t_dir = to_anchor.and_then(exit_dir).unwrap_or((-1.0, 0.0));

    let c1 = (from.0 + f_dir.0 * handle, from.1 + f_dir.1 * handle);
    let c2 = (to.0 + t_dir.0 * handle, to.1 + t_dir.1 * handle);
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
    fn east_to_west_ports_exit_horizontally() {
        // Both endpoints on the same y. Stubs collapse the corner.
        let p = orthogonal_with_anchors(
            (0.0, 50.0),
            (200.0, 50.0),
            Some(PortAnchor::East(0.5)),
            Some(PortAnchor::West(0.5)),
        );
        assert_eq!(
            p,
            vec![(0.0, 50.0), (20.0, 50.0), (180.0, 50.0), (200.0, 50.0)]
        );
    }

    #[test]
    fn north_to_south_ports_exit_vertically() {
        // Same x — the wire is a clean vertical run with stubs.
        let p = orthogonal_with_anchors(
            (50.0, 0.0),
            (50.0, 200.0),
            Some(PortAnchor::North(0.5)),
            Some(PortAnchor::South(0.5)),
        );
        assert_eq!(
            p,
            vec![(50.0, 0.0), (50.0, -20.0), (50.0, 220.0), (50.0, 200.0)]
        );
    }

    #[test]
    fn east_then_south_mixes_via_one_corner() {
        // East exit + South exit — single L-bend at (p1.x, p2.y).
        let p = orthogonal_with_anchors(
            (0.0, 50.0),
            (50.0, 200.0),
            Some(PortAnchor::East(0.5)),
            Some(PortAnchor::South(0.5)),
        );
        assert_eq!(
            p,
            vec![
                (0.0, 50.0),
                (20.0, 50.0),
                (20.0, 220.0),
                (50.0, 220.0),
                (50.0, 200.0),
            ]
        );
    }

    #[test]
    fn missing_anchor_falls_back_to_mid_x() {
        let p = orthogonal_with_anchors((0.0, 0.0), (100.0, 40.0), None, None);
        assert_eq!(p, vec![(0.0, 0.0), (50.0, 0.0), (50.0, 40.0), (100.0, 40.0)]);
    }

    #[test]
    fn free_anchor_falls_back_to_mid_x() {
        let p = orthogonal_with_anchors(
            (0.0, 0.0),
            (100.0, 40.0),
            Some(PortAnchor::Free(0.5, 0.5)),
            Some(PortAnchor::West(0.5)),
        );
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
    fn bezier_with_anchors_exits_perpendicular_to_each_port_face() {
        // West-exit start, south-exit end: the first-step direction
        // should head in -x; the last-step direction (looking back
        // from `to`) should come from +y.
        let from = (200.0, 100.0);
        let to = (400.0, 400.0);
        let poly = bezier_with_anchors(
            from,
            to,
            Some(PortAnchor::West(0.5)),
            Some(PortAnchor::South(0.5)),
        );
        assert_eq!(poly.first().copied(), Some(from));
        assert_eq!(poly.last().copied(), Some(to));
        // Sample direction immediately leaving `from` — must have a
        // negative x component (west exit) and small y component.
        let second = poly[1];
        let leave = (second.0 - from.0, second.1 - from.1);
        assert!(leave.0 < 0.0, "west exit must move -x, got {leave:?}");
        // Direction approaching `to` — must come from +y side.
        let penult = poly[poly.len() - 2];
        let approach = (to.0 - penult.0, to.1 - penult.1);
        assert!(approach.1 < 0.0, "south entry must approach with -y delta, got {approach:?}");
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
