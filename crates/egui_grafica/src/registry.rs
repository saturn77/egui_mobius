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

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, MutexGuard};

use egui_mobius_reactive::Dynamic;

use crate::model::{
    CanvasSettings, Edge, EdgeId, EdgeOverlay, Node, NodeId, Overlay, Port, PortAnchor, PortId,
    Routing, Scene,
};

/// Owns the [`Scene`] and is the only legitimate place to mutate it.
#[derive(Clone)]
pub struct Registry {
    scene: Dynamic<Scene>,
    /// Bumped on every mutation. `Arc` so clones share one counter.
    generation: Arc<AtomicU64>,
}

impl Registry {
    /// Wrap a freshly-constructed scene.
    pub fn new(scene: Scene) -> Self {
        Self { scene: Dynamic::new(scene), generation: Arc::new(AtomicU64::new(0)) }
    }

    /// Wrap an already-reactive scene (e.g. one shared across citizens).
    pub fn from_dynamic(scene: Dynamic<Scene>) -> Self {
        Self { scene, generation: Arc::new(AtomicU64::new(0)) }
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
        self.mutate(|scene| {
            if let Some(node) = scene.nodes.iter_mut().find(|n| &n.id == id) {
                node.transform.position = position;
            }
        });
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
    /// rather than one per node. This is the path a multi-node drag uses.
    pub fn move_nodes(&self, moves: &[(NodeId, (f32, f32))]) {
        self.mutate(|scene| {
            for (id, position) in moves {
                if let Some(node) = scene.nodes.iter_mut().find(|n| &n.id == id) {
                    node.transform.position = *position;
                }
            }
        });
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
        let new_scene = {
            let mut guard: MutexGuard<'_, Scene> = self.scene.lock();
            f(&mut guard);
            guard.clone()
        };
        self.scene.set(new_scene);
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

// =============================================================================
// Helpers
// =============================================================================

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
    use crate::model::{NodeKind, Transform};

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
}
