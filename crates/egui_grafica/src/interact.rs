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
use crate::router::{edge_polyline, port_position_on_node};

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
// Interaction state machine
// =============================================================================
//
// Ported from simcore's router FSM. The states and the transition table are
// the same; the only adaptation for egui is that transitions are driven by
// per-frame `Response` polling (drag_started / dragged / drag_stopped) rather
// than by retained-mode press/move/release event callbacks.

/// What the canvas pointer is currently doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CanvasState {
    /// Resting — no gesture in progress.
    #[default]
    Idle,
    /// Panning the viewport.
    Panning,
    /// Moving the selected nodes.
    MovingNodes,
    /// Drawing a new connection from a source port.
    Connecting,
    /// Dragging a wire segment to re-route it (1 DOF, perpendicular).
    DraggingSegment,
    /// Dragging a wire's pivot vertex (2 DOF, free).
    DraggingWaypoint,
}

/// What the pointer pressed on — decides which gesture a press begins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTarget {
    Empty,
    NodeBody,
    Port,
    WireSegment,
    /// A pivot vertex on a hand-routed wire.
    Waypoint,
}

/// Events that drive FSM transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanvasEvent {
    Press,
    Release,
    Cancel,
}

/// The transition table — `(state, event, target) -> next state`. `None`
/// means the event is not valid in the current state and is ignored.
fn next_state(state: CanvasState, event: CanvasEvent, target: HitTarget) -> Option<CanvasState> {
    use CanvasEvent::*;
    use CanvasState::*;
    use HitTarget::*;
    match (state, event, target) {
        // From Idle, a press begins a gesture chosen by what was hit.
        (Idle, Press, Empty) => Some(Panning),
        (Idle, Press, NodeBody) => Some(MovingNodes),
        (Idle, Press, Port) => Some(Connecting),
        (Idle, Press, Waypoint) => Some(DraggingWaypoint),
        (Idle, Press, WireSegment) => Some(DraggingSegment),
        // Any active gesture ends on release or cancel.
        (s, Release, _) | (s, Cancel, _) if s != Idle => Some(Idle),
        _ => None,
    }
}

/// Runtime state for the canvas interaction FSM: the current state plus the
/// context for whatever gesture is active. Context is cleared on return to
/// [`CanvasState::Idle`], exactly as in simcore's `RouterFSMState`.
#[derive(Debug, Clone, Default)]
pub struct CanvasFsm {
    pub state: CanvasState,
    /// World point where the active gesture began.
    pub grab_world: (f32, f32),
    /// Live pointer position in world space (updated each drag frame).
    pub cursor_world: (f32, f32),
    /// `MovingNodes`: each dragged node's position at grab time.
    pub node_origins: Vec<(NodeId, (f32, f32))>,
    /// `Connecting`: the source port.
    pub connect_from: Option<(NodeId, PortId)>,
    /// `DraggingSegment` / `DraggingWaypoint`: the wire being re-routed.
    pub drag_edge: Option<EdgeId>,
    /// `DraggingSegment`: index of the dragged segment in `drag_origin_pts`.
    pub drag_segment: usize,
    /// `DraggingSegment`: whether the dragged segment is horizontal.
    pub drag_axis_horizontal: bool,
    /// `DraggingSegment`: the full point list at grab time, with endpoint
    /// stubs inserted so the grabbed segment has two movable interior ends.
    pub drag_origin_pts: Vec<(f32, f32)>,
    /// `DraggingWaypoint`: index into the wire's waypoint list.
    pub drag_waypoint: usize,
}

impl CanvasFsm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_idle(&self) -> bool {
        self.state == CanvasState::Idle
    }

    /// Drive a transition. Returns `true` if it was valid (state changed or a
    /// self-loop fired); `false` if the event is not legal in this state.
    pub fn dispatch(&mut self, event: CanvasEvent, target: HitTarget) -> bool {
        match next_state(self.state, event, target) {
            Some(next) => {
                self.state = next;
                if next == CanvasState::Idle {
                    self.clear_context();
                }
                true
            }
            None => false,
        }
    }

    /// Force the FSM back to `Idle` — e.g. on focus loss.
    pub fn force_idle(&mut self) {
        self.state = CanvasState::Idle;
        self.clear_context();
    }

    fn clear_context(&mut self) {
        self.node_origins.clear();
        self.connect_from = None;
        self.drag_edge = None;
        self.drag_segment = 0;
        self.drag_origin_pts.clear();
        self.drag_waypoint = 0;
    }
}

// =============================================================================
// Wire re-routing geometry
// =============================================================================

/// The pivot vertex on a hand-routed wire nearest `world`, within `radius`.
/// Only `Routing::Manual` wires have pivots. Returns `(edge id, waypoint
/// index)`.
pub fn hit_test_waypoint(scene: &Scene, world: (f32, f32), radius: f32) -> Option<(EdgeId, usize)> {
    let r2 = radius * radius;
    let mut best: Option<(f32, EdgeId, usize)> = None;
    for edge in &scene.edges {
        if let crate::model::Routing::Manual { waypoints } = &edge.routing {
            for (i, &(wx, wy)) in waypoints.iter().enumerate() {
                let d2 = (wx - world.0).powi(2) + (wy - world.1).powi(2);
                if d2 <= r2 && best.as_ref().is_none_or(|(bd, _, _)| d2 < *bd) {
                    best = Some((d2, edge.id.clone(), i));
                }
            }
        }
    }
    best.map(|(_, id, i)| (id, i))
}

/// A prepared segment drag — see [`prepare_segment_drag`].
pub struct SegmentDrag {
    /// The full point list, with endpoint stubs inserted.
    pub points: Vec<(f32, f32)>,
    /// Index of the grabbed segment within `points`.
    pub segment: usize,
    /// Whether the grabbed segment is horizontal (drag moves it in Y).
    pub horizontal: bool,
}

/// Prepare a segment drag. Given a wire's `polyline` and the `press` point,
/// returns the working point list — with a stub inserted if the grabbed
/// segment touched a pinned endpoint, so the segment has two movable interior
/// ends — plus the grabbed segment index and orientation.
pub fn prepare_segment_drag(polyline: &[(f32, f32)], press: (f32, f32)) -> Option<SegmentDrag> {
    if polyline.len() < 2 {
        return None;
    }
    let mut pts = polyline.to_vec();

    // Nearest segment to the press.
    let mut k = 0usize;
    let mut best = f32::INFINITY;
    for i in 0..pts.len() - 1 {
        let d = point_segment_distance(press, pts[i], pts[i + 1]);
        if d < best {
            best = d;
            k = i;
        }
    }

    // If the grabbed segment touches a pinned endpoint, clone that endpoint
    // into a new interior waypoint so the whole segment can move.
    if k == 0 {
        pts.insert(1, pts[0]);
        k = 1;
    }
    if k + 1 == pts.len() - 1 {
        let last = pts.len() - 1;
        pts.insert(last, pts[last]);
    }

    let horizontal = (pts[k].1 - pts[k + 1].1).abs() <= (pts[k].0 - pts[k + 1].0).abs();
    Some(SegmentDrag { points: pts, segment: k, horizontal })
}

/// Insert a pivot vertex into a wire's `polyline` at the point on it nearest
/// `click`. Returns the new interior waypoint list (the polyline minus its
/// pinned endpoints).
pub fn insert_pivot(polyline: &[(f32, f32)], click: (f32, f32)) -> Vec<(f32, f32)> {
    if polyline.len() < 2 {
        return Vec::new();
    }
    // Nearest segment, and the projected point on it.
    let mut k = 0usize;
    let mut best = f32::INFINITY;
    let mut pivot = polyline[0];
    for i in 0..polyline.len() - 1 {
        let p = project_onto_segment(click, polyline[i], polyline[i + 1]);
        let d = ((p.0 - click.0).powi(2) + (p.1 - click.1).powi(2)).sqrt();
        if d < best {
            best = d;
            k = i;
            pivot = p;
        }
    }
    let mut pts = polyline.to_vec();
    pts.insert(k + 1, pivot);
    // Interior points = the waypoint list.
    pts[1..pts.len() - 1].to_vec()
}

/// Closest point on segment `a`–`b` to `p`.
fn project_onto_segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    let (abx, aby) = (b.0 - a.0, b.1 - a.1);
    let len2 = abx * abx + aby * aby;
    let t = if len2 <= f32::EPSILON {
        0.0
    } else {
        (((p.0 - a.0) * abx + (p.1 - a.1) * aby) / len2).clamp(0.0, 1.0)
    };
    (a.0 + t * abx, a.1 + t * aby)
}

// =============================================================================
// Hit testing
// =============================================================================

/// Topmost (last-drawn) node whose exact shape contour contains `world`.
///
/// A cheap bounding-box test rejects obvious misses; survivors are confirmed
/// against the node's hypercurve [`Contour2`](hypercurve::Contour2), so a
/// click in the empty corner of a circle's or ellipse's bounding box does
/// not register as a hit.
pub fn hit_test_node(scene: &Scene, world: (f32, f32)) -> Option<NodeId> {
    scene
        .nodes
        .iter()
        .rev()
        .find(|n| node_bounds_contains(n, world) && crate::geometry::contour_contains(n, world))
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
        let Some(poly) = edge_polyline(scene, edge) else {
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
    fn hit_test_node_respects_circle_contour_not_just_bounding_box() {
        let mut scene = Scene::default();
        let mut circle = rect_node("c", (0.0, 0.0), (100.0, 100.0));
        circle.kind = NodeKind::Circle;
        scene.nodes.push(circle);

        // Centre of the circle — inside.
        assert_eq!(hit_test_node(&scene, (50.0, 50.0)), Some(NodeId("c".into())));
        // A bounding-box corner — inside the AABB but outside the circle.
        assert_eq!(hit_test_node(&scene, (3.0, 3.0)), None);
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
    fn fsm_press_chooses_gesture_by_hit_target() {
        let mut fsm = CanvasFsm::new();
        assert!(fsm.dispatch(CanvasEvent::Press, HitTarget::Port));
        assert_eq!(fsm.state, CanvasState::Connecting);
        assert!(fsm.dispatch(CanvasEvent::Release, HitTarget::Empty));
        assert_eq!(fsm.state, CanvasState::Idle);

        fsm.dispatch(CanvasEvent::Press, HitTarget::WireSegment);
        assert_eq!(fsm.state, CanvasState::DraggingSegment);
        fsm.dispatch(CanvasEvent::Release, HitTarget::Empty);
        assert_eq!(fsm.state, CanvasState::Idle);

        fsm.dispatch(CanvasEvent::Press, HitTarget::Empty);
        assert_eq!(fsm.state, CanvasState::Panning);
    }

    #[test]
    fn fsm_rejects_a_press_while_already_in_a_gesture() {
        let mut fsm = CanvasFsm::new();
        fsm.dispatch(CanvasEvent::Press, HitTarget::NodeBody);
        assert_eq!(fsm.state, CanvasState::MovingNodes);
        // A second press mid-gesture is not a legal transition.
        assert!(!fsm.dispatch(CanvasEvent::Press, HitTarget::Port));
        assert_eq!(fsm.state, CanvasState::MovingNodes);
    }

    #[test]
    fn fsm_clears_context_on_return_to_idle() {
        let mut fsm = CanvasFsm::new();
        fsm.dispatch(CanvasEvent::Press, HitTarget::Port);
        fsm.connect_from = Some((NodeId("n".into()), PortId("p".into())));
        fsm.dispatch(CanvasEvent::Release, HitTarget::Empty);
        assert!(fsm.connect_from.is_none(), "context must clear on Idle");
    }

    #[test]
    fn snap_rounds_to_nearest_multiple() {
        assert_eq!(snap_to_grid((23.0, 47.0), 10.0), (20.0, 50.0));
        assert_eq!(snap_to_grid((23.0, 47.0), 0.0), (23.0, 47.0));
    }
}
