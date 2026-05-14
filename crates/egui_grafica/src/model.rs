//! Pure data model for a canvas scene.
//!
//! No egui dependency lives in this module. Colors are hex strings,
//! positions are `(f32, f32)` tuples. The `render` module converts to
//! egui types at draw time; the `lang` module converts to/from the
//! `.canvas` DSL text form.
//!
//! Keeping this module renderer-agnostic is what lets a `Scene` outlive
//! any particular GUI toolkit.

use serde::{Deserialize, Serialize};

// =============================================================================
// IDs
// =============================================================================

/// Stable identifier for a node within a scene.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

/// Stable identifier for a port within a node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId(pub String);

/// Stable identifier for an edge within a scene.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub String);

// =============================================================================
// Scene
// =============================================================================

/// Root of a canvas document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub settings: CanvasSettings,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub groups: Vec<Group>,
}

/// Canvas-level settings: grid, snap, paper size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasSettings {
    pub grid_spacing: f32,
    pub snap_to_grid: bool,
    pub show_grid: bool,
    pub paper_size: Option<String>,
    pub paper_orientation: Option<String>,
    pub default_routing: Routing,
}

impl Default for CanvasSettings {
    fn default() -> Self {
        Self {
            grid_spacing: 10.0,
            snap_to_grid: true,
            show_grid: true,
            paper_size: None,
            paper_orientation: None,
            default_routing: Routing::Orthogonal,
        }
    }
}

// =============================================================================
// Node
// =============================================================================

/// A placed shape on the canvas with ports and styling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub transform: Transform,
    pub overlay: Overlay,
    pub ports: Vec<Port>,
}

/// The primitive shape backing a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
    Rect,
    Circle,
    Ellipse,
    /// A free-form path defined by a list of segments.
    Path(Vec<PathSegment>),
    /// A node that is itself a sub-scene (hierarchical canvas).
    Group(NodeId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PathSegment {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    BezierTo {
        cp1: (f32, f32),
        cp2: (f32, f32),
        to: (f32, f32),
    },
    Close,
}

/// Position, size, rotation (degrees).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: (0.0, 0.0),
            size: (60.0, 40.0),
            rotation: 0.0,
        }
    }
}

// =============================================================================
// Port
// =============================================================================

/// A connection point on the periphery of a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub kind: PortKind,
    pub anchor: PortAnchor,
    /// Optional type tag for typed-connection validation in node-graph mode
    /// (e.g. "Real", "Bus<32>", "color"). Block-diagram ports leave this `None`.
    pub data_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortKind {
    In,
    Out,
    Bidir,
    Untyped,
}

/// Where on the node's perimeter the port lives.
///
/// `North(0.5)` is top-center, `East(0.0)` is top-right-corner, `East(1.0)`
/// is bottom-right-corner. The `spread` keyword in the DSL is sugar for
/// evenly-distributed `t` values.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PortAnchor {
    North(f32),
    South(f32),
    East(f32),
    West(f32),
    /// Free attachment in local (unit-square) coordinates.
    Free(f32, f32),
}

// =============================================================================
// Edge
// =============================================================================

/// A connection between two ports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub from: (NodeId, PortId),
    pub to: (NodeId, PortId),
    pub routing: Routing,
    pub overlay: EdgeOverlay,
}

/// How an edge is routed between its endpoints.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Routing {
    #[default]
    Orthogonal,
    Bezier,
    Straight,
    /// Manually-specified segments: H(dx) / V(dy) deltas.
    Manual(Vec<RouteSegment>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RouteSegment {
    H(f32),
    V(f32),
}

/// Visual style for an edge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeOverlay {
    pub color: String,
    pub width: f32,
    pub line_style: LineStyle,
    pub arrow_head: ArrowHead,
    pub arrow_tail: ArrowHead,
    pub label: Option<String>,
}

impl Default for EdgeOverlay {
    fn default() -> Self {
        Self {
            color: "#374151".to_string(),
            width: 1.5,
            line_style: LineStyle::Solid,
            arrow_head: ArrowHead::Arrow,
            arrow_tail: ArrowHead::None,
            label: None,
        }
    }
}

// =============================================================================
// Overlay (node styling)
// =============================================================================

/// Visual style for a node — border, fill, optional text.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Overlay {
    pub border: Border,
    pub fill: Fill,
    pub text: Option<TextLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Border {
    pub color: String,
    pub width: f32,
    pub style: LineStyle,
}

impl Default for Border {
    fn default() -> Self {
        Self {
            color: "#1F2937".to_string(),
            width: 1.5,
            style: LineStyle::Solid,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub color: String,
    /// 0.0 = transparent, 1.0 = opaque.
    pub alpha: f32,
}

impl Default for Fill {
    fn default() -> Self {
        Self {
            color: "#FFFFFF".to_string(),
            alpha: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArrowHead {
    None,
    Arrow,
    Triangle,
    Diamond,
    Circle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextLabel {
    pub value: String,
    pub anchor: TextAnchor,
    pub font_family: String,
    pub font_size: f32,
    pub bold: bool,
    pub italic: bool,
    pub color: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAnchor {
    Center,
    TopCenter,
    BottomCenter,
    Left,
    Right,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

// =============================================================================
// Group (hierarchical structure)
// =============================================================================

/// A named collection of nodes that move and select together.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: NodeId,
    pub label: Option<String>,
    pub members: Vec<NodeId>,
}
