//! `Registry` — the backend-model demarcation for a canvas.
//!
//! The registry owns the `Scene` and exposes a coarse-grained, typed mutation
//! API. Every GUI edit goes through one of these methods; nothing reaches into
//! `Scene` fields directly. This keeps mutations observable, undoable, and
//! one-to-one mappable to a future JSON-RPC server without restructuring.
//!
//! Even though everything is in-process Rust today, the boundary matters:
//!
//! - **Single point of mutation.** All edits flow through `Registry`. Change
//!   notification, undo tracking, and validation all hook in here.
//! - **Reactive by default.** The scene is held in a `Dynamic<Scene>` so that
//!   property panels and the canvas itself observe edits without polling.
//! - **Future-proof.** When/if a TCP frontend story arrives, the JSON-RPC
//!   handlers wrap `Registry` methods one-to-one. The model doesn't move.
//!
//! ## Read access
//!
//! Use [`Registry::with_scene`] for borrowed access (no clone) or
//! [`Registry::scene`] when you need an owned snapshot. The renderer takes
//! `&Scene`, so `with_scene` is the natural pairing.
//!
//! ## Subscribing to changes
//!
//! `Registry::scene_dynamic()` returns the underlying `Dynamic<Scene>`;
//! callers can `.on_change(...)` to receive notifications.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use egui_mobius_reactive::Dynamic;

use crate::model::{
    CanvasSettings, Edge, EdgeEnd, EdgeEndSide, EdgeId, EdgeOverlay, Node, NodeId, Overlay, Port,
    PortAnchor, PortId, Routing, Scene,
};

/// Maximum number of undo snapshots retained. Older snapshots are
/// dropped from the front when the stack would grow past this.
const UNDO_CAP: usize = 100;

/// Owns the [`Scene`] and is the only legitimate place to mutate it.
#[derive(Clone)]
pub struct Registry {
    scene: Dynamic<Scene>,
    /// Bumped on every mutation. `Arc` so clones share one counter.
    generation: Arc<AtomicU64>,
    /// Pre-mutation snapshots, newest at the back. Cleared on every
    /// new mutation so the redo chain only survives until the user
    /// edits past the undo point.
    undo_stack: Arc<Mutex<Vec<Scene>>>,
    /// Snapshots popped off `undo_stack` by `undo()` — popped back on
    /// `redo()`. Cleared when a fresh mutation lands.
    redo_stack: Arc<Mutex<Vec<Scene>>>,
    /// While > 0, `mutate` doesn't push new undo snapshots — used to
    /// collapse keystroke-by-keystroke text edits and frame-by-frame
    /// drags into a single undoable step. The first call to
    /// `begin_undo_batch` takes one snapshot; nested calls just bump
    /// the depth.
    batch_depth: Arc<AtomicU32>,
}

impl Registry {
    /// Wrap a freshly-constructed scene.
    pub fn new(scene: Scene) -> Self {
        Self {
            scene: Dynamic::new(scene),
            generation: Arc::new(AtomicU64::new(0)),
            undo_stack: Arc::new(Mutex::new(Vec::new())),
            redo_stack: Arc::new(Mutex::new(Vec::new())),
            batch_depth: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Wrap an already-reactive scene (e.g. one shared across citizens).
    pub fn from_dynamic(scene: Dynamic<Scene>) -> Self {
        Self {
            scene,
            generation: Arc::new(AtomicU64::new(0)),
            undo_stack: Arc::new(Mutex::new(Vec::new())),
            redo_stack: Arc::new(Mutex::new(Vec::new())),
            batch_depth: Arc::new(AtomicU32::new(0)),
        }
    }

    // ─── Undo / redo ──────────────────────────────────────────────────────

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.lock().unwrap().is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.lock().unwrap().is_empty()
    }

    /// Restore the most recent pre-mutation snapshot. Pushes the
    /// current scene onto the redo stack so the user can replay
    /// forward.
    pub fn undo(&self) {
        let prev = self.undo_stack.lock().unwrap().pop();
        if let Some(prev) = prev {
            let current = self.scene.get();
            self.redo_stack.lock().unwrap().push(current);
            self.scene.set(prev);
            self.generation.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Replay the most recent undone snapshot. Pushes the current
    /// scene back onto the undo stack so a fresh undo retreats again.
    pub fn redo(&self) {
        let next = self.redo_stack.lock().unwrap().pop();
        if let Some(next) = next {
            let current = self.scene.get();
            self.undo_stack.lock().unwrap().push(current);
            self.scene.set(next);
            self.generation.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Open an undo batch. The first call (depth 0 → 1) snapshots the
    /// current scene; intervening `mutate` calls don't add new
    /// snapshots. Nested calls just deepen — only the outermost
    /// `end_undo_batch` re-enables snapshotting. Used to collapse
    /// text-edit keystrokes and drag frames into one undoable step.
    pub fn begin_undo_batch(&self) {
        let prev = self.batch_depth.fetch_add(1, Ordering::Relaxed);
        if prev == 0 {
            let snap = self.scene.get();
            let mut undo = self.undo_stack.lock().unwrap();
            undo.push(snap);
            if undo.len() > UNDO_CAP {
                undo.remove(0);
            }
            self.redo_stack.lock().unwrap().clear();
        }
    }

    /// Close an undo batch opened by [`Self::begin_undo_batch`].
    pub fn end_undo_batch(&self) {
        let prev = self.batch_depth.load(Ordering::Relaxed);
        if prev > 0 {
            self.batch_depth.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Monotonic counter, bumped on every mutation. Consumers cache
    /// derived data — e.g. the GPU instance buffers — keyed by this
    /// value and rebuild only when it changes.
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    /// Underlying reactive container for subscription-based observers.
    pub fn scene_dynamic(&self) -> &Dynamic<Scene> {
        &self.scene
    }

    /// Borrowed read access — preferred over [`Self::scene`] when you just
    /// need to read fields. Holds the internal lock for the closure's
    /// duration.
    pub fn with_scene<R>(&self, f: impl FnOnce(&Scene) -> R) -> R {
        let guard = self.scene.lock();
        f(&guard)
    }

    /// Cloned snapshot. Use when you need to hand the scene to code that can't
    /// hold the lock (cross-thread serialization, etc.).
    pub fn scene(&self) -> Scene {
        self.scene.get()
    }

    /// Replace the entire scene — e.g. after loading a `.canvas` file.
    pub fn set_scene(&self, scene: Scene) {
        self.scene.set(scene);
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    // ─── Mutations ────────────────────────────────────────────────────────

    /// Insert a node. If a node with this id already exists, it is replaced.
    pub fn add_node(&self, node: Node) {
        self.mutate(|scene| {
            if let Some(slot) = scene.nodes.iter_mut().find(|n| n.id == node.id) {
                *slot = node;
            } else {
                scene.nodes.push(node);
            }
        });
    }

    pub fn remove_node(&self, id: &NodeId) {
        self.mutate(|scene| {
            scene.nodes.retain(|n| &n.id != id);
            // Drop edges that touch the removed node — otherwise the scene
            // has dangling references and the renderer silently skips them.
            scene.edges.retain(|e| e.from.node_id() != Some(id) && e.to.node_id() != Some(id));
        });
    }

    pub fn move_node(&self, id: &NodeId, position: (f32, f32)) {
        // Single-node moves go through the multi-node path so the
        // adjacent-waypoint translation kicks in there too.
        self.move_nodes(&[(id.clone(), position)]);
    }

    /// Append a port to a node.
    pub fn add_port(&self, node: &NodeId, port: Port) {
        self.mutate(|scene| {
            if let Some(n) = scene.nodes.iter_mut().find(|n| &n.id == node) {
                n.ports.push(port);
            }
        });
    }

    /// Move a port to a new anchor on its node's perimeter. Wires attached to
    /// the port follow automatically — they reference it relationally.
    pub fn set_port_anchor(&self, node: &NodeId, port: &PortId, anchor: PortAnchor) {
        self.mutate(|scene| {
            if let Some(n) = scene.nodes.iter_mut().find(|n| &n.id == node)
                && let Some(p) = n.ports.iter_mut().find(|p| &p.id == port)
            {
                p.anchor = anchor;
            }
        });
    }

    /// Move several nodes in one mutation — a single change notification
    /// rather than one per node. The path a multi-node drag uses.
    ///
    /// Manual-wire waypoints adjacent to a moved port translate with
    /// the port: a horizontal port-entry segment stays horizontal, a
    /// vertical one stays vertical. Without this, dragging a node
    /// would leave its wires' "elbows" frozen at the old position.
    pub fn move_nodes(&self, moves: &[(NodeId, (f32, f32))]) {
        self.mutate(|scene| {
            // Per-node delta (new − old). Skip ids that don't exist.
            let deltas: Vec<(NodeId, (f32, f32))> = moves
                .iter()
                .filter_map(|(id, new_pos)| {
                    let n = scene.nodes.iter().find(|n| &n.id == id)?;
                    let old = n.transform.position;
                    Some((id.clone(), (new_pos.0 - old.0, new_pos.1 - old.1)))
                })
                .collect();

            // Pre-move snapshot of every edge's endpoint world positions —
            // needed to decide segment orientation, taken before we
            // start mutating. Covers Straight wires with Free ends too,
            // not just Manual.
            type PortSnap = (EdgeId, Option<(f32, f32)>, Option<(f32, f32)>);
            let port_snapshots: Vec<PortSnap> = scene
                .edges
                .iter()
                .map(|e| {
                    (
                        e.id.clone(),
                        crate::router::edge_end_position(scene, &e.from),
                        crate::router::edge_end_position(scene, &e.to),
                    )
                })
                .collect();

            // Translate the in-wire point adjacent to each moved port.
            // For Manual routing that's the first / last interior
            // waypoint. For Straight (or any other routing without
            // interior waypoints), the adjacent point is the OTHER
            // endpoint — translated only if it's a Free dangling point,
            // so orphaned segments keep their orientation.
            for edge in scene.edges.iter_mut() {
                let from_delta = edge.from.node_id().and_then(|n| {
                    deltas.iter().find(|(id, _)| id == n).map(|(_, d)| *d)
                });
                let to_delta = edge.to.node_id().and_then(|n| {
                    deltas.iter().find(|(id, _)| id == n).map(|(_, d)| *d)
                });
                if from_delta.is_none() && to_delta.is_none() {
                    continue;
                }
                // Self-loop on a single moved node: translate every
                // waypoint by the (shared) delta — the whole wire moves.
                let same_node = matches!(
                    (&edge.from, &edge.to),
                    (EdgeEnd::Port(a, _), EdgeEnd::Port(b, _)) if a == b,
                );
                if same_node && let Some((dx, dy)) = from_delta {
                    if let Routing::Manual { waypoints } = &mut edge.routing {
                        for w in waypoints.iter_mut() {
                            w.0 += dx;
                            w.1 += dy;
                        }
                    }
                    continue;
                }

                let (from_port, to_port) = port_snapshots
                    .iter()
                    .find(|(id, _, _)| id == &edge.id)
                    .map(|(_, f, t)| (*f, *t))
                    .unwrap_or((None, None));

                if let Some(delta) = from_delta
                    && let Some(port) = from_port
                {
                    let moved_waypoint = if let Routing::Manual { waypoints } = &mut edge.routing
                        && let Some(first) = waypoints.first_mut()
                    {
                        translate_adjacent(first, port, delta);
                        true
                    } else {
                        false
                    };
                    if !moved_waypoint {
                        try_translate_free(&mut edge.to, port, delta);
                    }
                }
                if let Some(delta) = to_delta
                    && let Some(port) = to_port
                {
                    let moved_waypoint = if let Routing::Manual { waypoints } = &mut edge.routing
                        && let Some(last) = waypoints.last_mut()
                    {
                        translate_adjacent(last, port, delta);
                        true
                    } else {
                        false
                    };
                    if !moved_waypoint {
                        try_translate_free(&mut edge.from, port, delta);
                    }
                }
            }

            // Apply the new positions.
            for (id, position) in moves {
                if let Some(node) = scene.nodes.iter_mut().find(|n| &n.id == id) {
                    node.transform.position = *position;
                }
            }
        });
    }

    /// Set a node's position *and* size in one mutation. The resize
    /// drag uses this to avoid the two extra generation bumps (and
    /// GPU re-uploads) that calling `move_node` + `resize_node`
    /// separately would produce per drag frame.
    pub fn set_node_transform(&self, id: &NodeId, position: (f32, f32), size: (f32, f32)) {
        self.mutate(|scene| {
            if let Some(node) = scene.nodes.iter_mut().find(|n| &n.id == id) {
                node.transform.position = position;
                node.transform.size = size;
            }
        });
    }

    /// Snap every node in `ids` to the nearest grid intersection.
    /// Routes through [`Self::move_nodes`] so the adjacent-waypoint
    /// follow logic stays consistent — wires connected to moved nodes
    /// drag their elbows with them. No-op when `grid_spacing <= 0`.
    pub fn align_selection_to_grid(&self, ids: &[NodeId]) {
        let spacing = self.with_scene(|s| s.settings.grid_spacing);
        if spacing <= 0.0 || ids.is_empty() {
            return;
        }
        let moves: Vec<(NodeId, (f32, f32))> = self.with_scene(|s| {
            ids.iter()
                .filter_map(|id| {
                    let node = s.nodes.iter().find(|n| &n.id == id)?;
                    let (x, y) = node.transform.position;
                    let snapped = (
                        (x / spacing).round() * spacing,
                        (y / spacing).round() * spacing,
                    );
                    if snapped == (x, y) {
                        None
                    } else {
                        Some((id.clone(), snapped))
                    }
                })
                .collect()
        });
        if !moves.is_empty() {
            self.move_nodes(&moves);
        }
    }

    pub fn resize_node(&self, id: &NodeId, size: (f32, f32)) {
        self.mutate(|scene| {
            if let Some(node) = scene.nodes.iter_mut().find(|n| &n.id == id) {
                node.transform.size = size;
            }
        });
    }

    pub fn update_node_overlay(&self, id: &NodeId, overlay: Overlay) {
        self.mutate(|scene| {
            if let Some(node) = scene.nodes.iter_mut().find(|n| &n.id == id) {
                node.overlay = overlay;
            }
        });
    }

    pub fn add_edge(&self, edge: Edge) {
        self.mutate(|scene| {
            if let Some(slot) = scene.edges.iter_mut().find(|e| e.id == edge.id) {
                *slot = edge;
            } else {
                scene.edges.push(edge);
            }
        });
    }

    pub fn remove_edge(&self, id: &EdgeId) {
        self.mutate(|scene| {
            scene.edges.retain(|e| &e.id != id);
        });
    }

    pub fn update_edge_routing(&self, id: &EdgeId, routing: Routing) {
        self.mutate(|scene| {
            if let Some(edge) = scene.edges.iter_mut().find(|e| &e.id == id) {
                edge.routing = routing;
            }
        });
    }

    /// Extend a wire by placing a new segment from one of its endpoints,
    /// preserving the existing endpoint as an interior waypoint. Used
    /// when the user drags a dangling `EdgeEnd::Free` to grow the wire
    /// onward — the cut point stays put, a new segment is added.
    ///
    /// Only meaningful when the side is currently `EdgeEnd::Free`. The
    /// new end may be either a port (reconnecting the wire through the
    /// preserved waypoint) or a fresh free point further out.
    pub fn extend_free_end(&self, id: &EdgeId, side: EdgeEndSide, new_end: EdgeEnd) {
        self.mutate(|scene| {
            let Some(edge) = scene.edges.iter_mut().find(|e| &e.id == id) else {
                return;
            };
            let current = match side {
                EdgeEndSide::From => &edge.from,
                EdgeEndSide::To => &edge.to,
            };
            let anchor = match current {
                EdgeEnd::Port(_, _) => return,
                EdgeEnd::Free(x, y) => (*x, *y),
            };
            // The old Free becomes the adjacent interior waypoint.
            match &mut edge.routing {
                Routing::Manual { waypoints } => match side {
                    EdgeEndSide::From => waypoints.insert(0, anchor),
                    EdgeEndSide::To => waypoints.push(anchor),
                },
                // Any auto-routing freezes to a Manual route with the
                // one preserved waypoint at the cut.
                _ => edge.routing = Routing::Manual { waypoints: vec![anchor] },
            }
            match side {
                EdgeEndSide::From => edge.from = new_end,
                EdgeEndSide::To => edge.to = new_end,
            }
        });
    }

    /// Reattach (or move) one end of a wire — used to drag a dangling
    /// `EdgeEnd::Free` endpoint onto a port (`EdgeEnd::Port`), or to
    /// move it to a new free position during the drag.
    pub fn update_edge_end(&self, id: &EdgeId, side: EdgeEndSide, end: EdgeEnd) {
        self.mutate(|scene| {
            if let Some(edge) = scene.edges.iter_mut().find(|e| &e.id == id) {
                match side {
                    EdgeEndSide::From => edge.from = end,
                    EdgeEndSide::To => edge.to = end,
                }
            }
        });
    }

    pub fn update_edge_overlay(&self, id: &EdgeId, overlay: EdgeOverlay) {
        self.mutate(|scene| {
            if let Some(edge) = scene.edges.iter_mut().find(|e| &e.id == id) {
                edge.overlay = overlay;
            }
        });
    }

    pub fn update_settings(&self, settings: CanvasSettings) {
        self.mutate(|scene| {
            scene.settings = settings;
        });
    }

    /// Toggle [`CanvasSettings::show_grid`]. Returns the new value.
    pub fn toggle_grid(&self) -> bool {
        let mut new_value = false;
        self.mutate(|scene| {
            scene.settings.show_grid = !scene.settings.show_grid;
            new_value = scene.settings.show_grid;
        });
        new_value
    }

    /// Mirror every node about the scene's horizontal centerline (flip Y).
    /// Equivalent to "mirror about the X axis" in math convention.
    pub fn mirror_scene_about_x(&self) {
        self.mutate(|scene| {
            if let Some(c) = scene_world_center(scene) {
                for node in &mut scene.nodes {
                    let h = node.transform.size.1;
                    node.transform.position.1 = 2.0 * c.1 - node.transform.position.1 - h;
                }
            }
        });
    }

    /// Mirror every node about the scene's vertical centerline (flip X).
    /// Equivalent to "mirror about the Y axis" in math convention.
    pub fn mirror_scene_about_y(&self) {
        self.mutate(|scene| {
            if let Some(c) = scene_world_center(scene) {
                for node in &mut scene.nodes {
                    let w = node.transform.size.0;
                    node.transform.position.0 = 2.0 * c.0 - node.transform.position.0 - w;
                }
            }
        });
    }

    /// Rotate every node 90° clockwise around the scene center. Each node's
    /// width and height are swapped so the bounding box stays axis-aligned.
    pub fn rotate_scene_90_cw(&self) {
        self.mutate(|scene| {
            if let Some(c) = scene_world_center(scene) {
                for node in &mut scene.nodes {
                    let (cx_n, cy_n) = (
                        node.transform.position.0 + node.transform.size.0 * 0.5,
                        node.transform.position.1 + node.transform.size.1 * 0.5,
                    );
                    // 90° CW in screen coords (y-down): (dx, dy) -> (-dy, dx)
                    let dx = cx_n - c.0;
                    let dy = cy_n - c.1;
                    let new_cx_n = c.0 - dy;
                    let new_cy_n = c.1 + dx;
                    let (w, h) = (node.transform.size.0, node.transform.size.1);
                    node.transform.size = (h, w);
                    node.transform.position = (new_cx_n - h * 0.5, new_cy_n - w * 0.5);
                }
            }
        });
    }

    // ─── Internals ────────────────────────────────────────────────────────

    /// Hold the lock, mutate, drop, then trigger change notification. We
    /// can't reach the notifier list directly through `Dynamic`, so we
    /// snapshot-clone post-mutation and call `set` — a deliberate trade-off:
    /// scenes large enough for this clone to matter want a more
    /// targeted notification path (per-node Dynamic), which is a later
    /// refactor.
    fn mutate(&self, f: impl FnOnce(&mut Scene)) {
        // Pre-mutation snapshot, captured outside the lock so we hold
        // the scene lock as briefly as possible.
        let before = if self.batch_depth.load(Ordering::Relaxed) == 0 {
            Some(self.scene.get())
        } else {
            None
        };
        let new_scene = {
            let mut guard: MutexGuard<'_, Scene> = self.scene.lock();
            f(&mut guard);
            guard.clone()
        };
        if let Some(before) = before {
            let mut undo = self.undo_stack.lock().unwrap();
            undo.push(before);
            if undo.len() > UNDO_CAP {
                undo.remove(0);
            }
            // Any fresh mutation invalidates the redo branch — the
            // user has stepped off the previous timeline.
            self.redo_stack.lock().unwrap().clear();
        }
        self.scene.set(new_scene);
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Translate a wire's [`EdgeEnd::Free`] dangling endpoint with the
/// same axis-preserving rule [`translate_adjacent`] applies to
/// waypoints. A no-op when the end is anchored to a port.
fn try_translate_free(end: &mut EdgeEnd, port: (f32, f32), delta: (f32, f32)) {
    if let EdgeEnd::Free(x, y) = end {
        let mut wp = (*x, *y);
        translate_adjacent(&mut wp, port, delta);
        *x = wp.0;
        *y = wp.1;
    }
}

/// Translate a Manual waypoint by `delta`, but only along the axis that
/// keeps the segment `port → waypoint` perpendicular to whichever
/// cardinal axis it was on. A horizontal segment tracks the port's
/// y-component, a vertical one tracks x, a clearly-diagonal one
/// translates by the full delta.
fn translate_adjacent(
    waypoint: &mut (f32, f32),
    port: (f32, f32),
    delta: (f32, f32),
) {
    let seg = (waypoint.0 - port.0, waypoint.1 - port.1);
    let dx_abs = seg.0.abs();
    let dy_abs = seg.1.abs();
    if dy_abs < dx_abs * 0.1 {
        // Horizontal segment — only update Y so it stays horizontal.
        waypoint.1 += delta.1;
    } else if dx_abs < dy_abs * 0.1 {
        // Vertical segment — only update X so it stays vertical.
        waypoint.0 += delta.0;
    } else {
        waypoint.0 += delta.0;
        waypoint.1 += delta.1;
    }
}

fn scene_world_center(scene: &Scene) -> Option<(f32, f32)> {
    let bounds = crate::render::scene_bounds(scene)?;
    Some((bounds.center().x, bounds.center().y))
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        EdgeEnd, NodeKind, Port, PortAnchor, PortKind, Transform,
    };

    fn node_rect(id: &str, pos: (f32, f32), size: (f32, f32)) -> Node {
        Node {
            id: NodeId(id.to_string()),
            kind: NodeKind::Rect,
            transform: Transform { position: pos, size, rotation: 0.0 },
            overlay: Overlay::default(),
            ports: vec![],
        }
    }

    #[test]
    fn add_and_move_node() {
        let reg = Registry::new(Scene::default());
        reg.add_node(node_rect("a", (0.0, 0.0), (10.0, 10.0)));
        reg.move_node(&NodeId("a".to_string()), (20.0, 30.0));
        let scene = reg.scene();
        assert_eq!(scene.nodes[0].transform.position, (20.0, 30.0));
    }

    #[test]
    fn remove_node_cascades_to_attached_edges() {
        let reg = Registry::new(Scene::default());
        reg.add_node(node_rect("a", (0.0, 0.0), (10.0, 10.0)));
        reg.add_node(node_rect("b", (50.0, 0.0), (10.0, 10.0)));
        reg.add_edge(Edge {
            id: EdgeId("e".to_string()),
            from: crate::model::EdgeEnd::Port(NodeId("a".to_string()), crate::model::PortId("x".to_string())),
            to: crate::model::EdgeEnd::Port(NodeId("b".to_string()), crate::model::PortId("y".to_string())),
            routing: Routing::default(),
            overlay: EdgeOverlay::default(),
        });
        reg.remove_node(&NodeId("a".to_string()));
        let scene = reg.scene();
        assert_eq!(scene.nodes.len(), 1);
        assert!(scene.edges.is_empty(), "edges attached to removed node should be dropped");
    }

    #[test]
    fn add_node_with_existing_id_replaces() {
        let reg = Registry::new(Scene::default());
        reg.add_node(node_rect("a", (0.0, 0.0), (10.0, 10.0)));
        reg.add_node(node_rect("a", (100.0, 100.0), (20.0, 20.0)));
        let scene = reg.scene();
        assert_eq!(scene.nodes.len(), 1);
        assert_eq!(scene.nodes[0].transform.position, (100.0, 100.0));
    }

    #[test]
    fn moving_a_node_drags_its_adjacent_manual_waypoint_along() {
        // Two boards on the same horizontal band, manually-routed L
        // between them: (a.east)=(100,50) → (150,50) → (150,150) →
        // (200,150)=(b.west, with b at y=100). The entry into B is a
        // horizontal run.
        let reg = Registry::new(Scene::default());
        let mut a = node_rect("a", (0.0, 0.0), (100.0, 100.0));
        a.ports.push(Port {
            id: PortId("pa".into()),
            name: "pa".into(),
            kind: PortKind::Out,
            anchor: PortAnchor::East(0.5),
            data_type: None,
        });
        let mut b = node_rect("b", (200.0, 100.0), (100.0, 100.0));
        b.ports.push(Port {
            id: PortId("pb".into()),
            name: "pb".into(),
            kind: PortKind::In,
            anchor: PortAnchor::West(0.5),
            data_type: None,
        });
        reg.add_node(a);
        reg.add_node(b);
        reg.add_edge(Edge {
            id: EdgeId("e".into()),
            from: EdgeEnd::Port(NodeId("a".into()), PortId("pa".into())),
            to: EdgeEnd::Port(NodeId("b".into()), PortId("pb".into())),
            routing: Routing::Manual { waypoints: vec![(150.0, 50.0), (150.0, 150.0)] },
            overlay: EdgeOverlay::default(),
        });

        // Move B up so its west port goes from (200, 150) to (200, 100).
        reg.move_node(&NodeId("b".into()), (200.0, 50.0));

        let scene = reg.scene();
        let edge = &scene.edges[0];
        let Routing::Manual { waypoints } = &edge.routing else {
            panic!("edge should still be Manual");
        };
        // First waypoint (sense_board side) is unchanged.
        assert_eq!(waypoints[0], (150.0, 50.0));
        // Second waypoint (B-adjacent) tracks B's Y so the horizontal
        // entry stays horizontal — y followed by -50, x unchanged.
        assert!((waypoints[1].1 - 100.0).abs() < 1e-3, "w2.y = {}", waypoints[1].1);
        assert_eq!(waypoints[1].0, 150.0);
    }

    #[test]
    fn extend_free_end_preserves_the_old_position_as_a_waypoint() {
        // I-shaped survivor: Port(A) — Free(corner), Straight. Extend
        // the Free end onward: the corner becomes a waypoint, the to
        // end is the new point. Original geometry stays put.
        let reg = Registry::new(Scene::default());
        let mut a = node_rect("a", (0.0, 0.0), (100.0, 100.0));
        a.ports.push(Port {
            id: PortId("pa".into()),
            name: "pa".into(),
            kind: PortKind::Out,
            anchor: PortAnchor::East(0.5),
            data_type: None,
        });
        reg.add_node(a);
        reg.add_edge(Edge {
            id: EdgeId("e".into()),
            from: EdgeEnd::Port(NodeId("a".into()), PortId("pa".into())),
            to: EdgeEnd::Free(200.0, 50.0),
            routing: Routing::Straight,
            overlay: EdgeOverlay::default(),
        });

        reg.extend_free_end(
            &EdgeId("e".into()),
            EdgeEndSide::To,
            EdgeEnd::Free(200.0, 200.0),
        );

        let edge = &reg.scene().edges[0];
        let Routing::Manual { waypoints } = &edge.routing else {
            panic!("Straight should have collapsed into Manual on extend");
        };
        // Old corner kept as the new waypoint, in its original position.
        assert_eq!(waypoints, &vec![(200.0, 50.0)]);
        // New to-end is the released point.
        assert!(matches!(edge.to, EdgeEnd::Free(x, y) if x == 200.0 && y == 200.0));
        // From end untouched.
        assert!(matches!(&edge.from, EdgeEnd::Port(n, _) if n.0 == "a"));
    }

    #[test]
    fn extending_from_a_free_end_onto_a_port_reconnects_through_the_anchor() {
        // The Free anchor is preserved as a waypoint even when the
        // released-on target is a port — reconnect routes *through*
        // the cut, not over it.
        let reg = Registry::new(Scene::default());
        reg.add_edge(Edge {
            id: EdgeId("e".into()),
            from: EdgeEnd::Free(50.0, 50.0),
            to: EdgeEnd::Free(120.0, 60.0),
            routing: Routing::Manual { waypoints: vec![(80.0, 50.0)] },
            overlay: EdgeOverlay::default(),
        });

        reg.extend_free_end(
            &EdgeId("e".into()),
            EdgeEndSide::To,
            EdgeEnd::Port(NodeId("b".into()), PortId("pb".into())),
        );

        let edge = &reg.scene().edges[0];
        let Routing::Manual { waypoints } = &edge.routing else {
            panic!()
        };
        // New waypoint appended at the previous Free-to position.
        assert_eq!(waypoints, &vec![(80.0, 50.0), (120.0, 60.0)]);
        assert!(matches!(&edge.to, EdgeEnd::Port(n, _) if n.0 == "b"));
    }

    #[test]
    fn moving_a_node_drags_the_orphan_segments_free_end_along() {
        // An orphaned wire-segment survivor: Routing::Straight from a
        // Free dangling point to a port, with no waypoints. The Free
        // end must follow the port to keep the segment's orientation.
        let reg = Registry::new(Scene::default());
        let mut b = node_rect("b", (200.0, 100.0), (100.0, 100.0));
        b.ports.push(Port {
            id: PortId("pb".into()),
            name: "pb".into(),
            kind: PortKind::In,
            anchor: PortAnchor::West(0.5), // world (200, 150)
            data_type: None,
        });
        reg.add_node(b);
        reg.add_edge(Edge {
            id: EdgeId("orphan".into()),
            from: EdgeEnd::Free(120.0, 150.0), // horizontal entry into B
            to: EdgeEnd::Port(NodeId("b".into()), PortId("pb".into())),
            routing: Routing::Straight,
            overlay: EdgeOverlay::default(),
        });

        // Move B up so its port goes from (200, 150) to (200, 100).
        reg.move_node(&NodeId("b".into()), (200.0, 50.0));

        let scene = reg.scene();
        let from = &scene.edges[0].from;
        let EdgeEnd::Free(fx, fy) = from else {
            panic!("from should still be Free");
        };
        // The horizontal segment must keep its orientation: x stays,
        // y follows the port (-50).
        assert_eq!(*fx, 120.0, "free x must not drift on a vertical drag");
        assert!((fy - 100.0).abs() < 1e-3, "free y = {} expected 100", fy);
    }

    #[test]
    fn undo_redo_round_trip_on_node_move() {
        let reg = Registry::new(Scene::default());
        reg.add_node(node_rect("a", (10.0, 20.0), (40.0, 40.0)));
        assert!(reg.can_undo(), "the add_node mutation should be undoable");
        assert!(!reg.can_redo());

        reg.move_node(&NodeId("a".into()), (100.0, 200.0));
        assert_eq!(reg.scene().nodes[0].transform.position, (100.0, 200.0));

        reg.undo();
        assert_eq!(reg.scene().nodes[0].transform.position, (10.0, 20.0));
        assert!(reg.can_redo());

        reg.redo();
        assert_eq!(reg.scene().nodes[0].transform.position, (100.0, 200.0));
    }

    #[test]
    fn undo_batch_collapses_many_mutations_into_one_step() {
        // Simulates a drag: many move_node calls inside one
        // begin/end_undo_batch pair should be a single undo step,
        // reverting all the way to the position before the drag.
        let reg = Registry::new(Scene::default());
        reg.add_node(node_rect("a", (0.0, 0.0), (40.0, 40.0)));

        reg.begin_undo_batch();
        for i in 1..=10 {
            reg.move_node(&NodeId("a".into()), (i as f32 * 10.0, 0.0));
        }
        reg.end_undo_batch();

        assert_eq!(reg.scene().nodes[0].transform.position, (100.0, 0.0));

        reg.undo();
        assert_eq!(
            reg.scene().nodes[0].transform.position,
            (0.0, 0.0),
            "a single undo should rewind the entire batch",
        );
    }

    #[test]
    fn fresh_mutation_invalidates_the_redo_branch() {
        let reg = Registry::new(Scene::default());
        reg.add_node(node_rect("a", (0.0, 0.0), (40.0, 40.0)));
        reg.move_node(&NodeId("a".into()), (50.0, 0.0));
        reg.undo();
        assert!(reg.can_redo());

        // Branching mutation: redo branch must drop.
        reg.move_node(&NodeId("a".into()), (5.0, 5.0));
        assert!(!reg.can_redo());
    }
}
