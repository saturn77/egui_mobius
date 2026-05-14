//! Interaction layer: selection, drag, port-to-port connect, marquee,
//! snap-to-grid, undo/redo.
//!
//! Operates on a `Scene` through edit commands so that every user
//! action is serializable, undoable, and round-trip-stable with the
//! `.canvas` DSL.
//!
//! Not yet implemented — sketched API:
//!
//! ```ignore
//! pub enum Command {
//!     MoveNode { id: NodeId, delta: (f32, f32) },
//!     ResizeNode { id: NodeId, new_size: (f32, f32) },
//!     SetOverlay { id: NodeId, overlay: Overlay },
//!     ConnectPorts { from: (NodeId, PortId), to: (NodeId, PortId) },
//!     DeleteNode { id: NodeId },
//!     DeleteEdge { id: EdgeId },
//! }
//!
//! pub fn apply(scene: &mut Scene, cmd: &Command);
//! pub fn undo (scene: &mut Scene, cmd: &Command);
//! ```

/// Selection state for the canvas. Stub.
#[derive(Debug, Default, Clone)]
pub struct Selection {
    pub nodes: Vec<crate::model::NodeId>,
    pub edges: Vec<crate::model::EdgeId>,
}
