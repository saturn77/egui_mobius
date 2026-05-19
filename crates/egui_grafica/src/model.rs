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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub name: String,
    pub settings: CanvasSettings,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub groups: Vec<Group>,
}

/// Canvas-level settings: grid, snap, paper size, routing default, units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CanvasSettings {
    /// Grid step in world units (the value displayed in the ribbon labelled
    /// with `grid_units`).
    pub grid_spacing: f32,
    pub snap_to_grid: bool,
    pub show_grid: bool,
    pub grid_style: GridStyle,
    /// Diameter of dot markers when `grid_style == Dots`, in world units.
    pub dot_size: f32,
    /// Display unit for ribbon labels. Purely cosmetic — the underlying math
    /// always uses world units. A user who picks `Millimeters` is declaring
    /// "I want to read these numbers as mm"; whether 1 world unit equals
    /// 1 mm depends on the user's mental model.
    pub grid_units: GridUnits,
    pub paper_size: Option<String>,
    pub paper_orientation: Option<String>,
    pub default_routing: Routing,
    /// Canvas background tone.
    pub background: CanvasBackground,
}

impl Default for CanvasSettings {
    fn default() -> Self {
        Self {
            grid_spacing: 10.0,
            snap_to_grid: true,
            show_grid: true,
            grid_style: GridStyle::Lines,
            dot_size: 2.0,
            grid_units: GridUnits::Pixels,
            paper_size: None,
            paper_orientation: None,
            default_routing: Routing::Orthogonal,
            background: CanvasBackground::Light,
        }
    }
}

/// Canvas background preset. The renderer maps each to a fill colour and a
/// contrasting grid tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanvasBackground {
    /// Near-white.
    Light,
    /// Soft grey — easy on the eyes.
    Slate,
    /// Dark blue-grey.
    Charcoal,
    /// Near-black.
    Dark,
}

impl CanvasBackground {
    pub fn label(self) -> &'static str {
        match self {
            CanvasBackground::Light => "Light",
            CanvasBackground::Slate => "Slate",
            CanvasBackground::Charcoal => "Charcoal",
            CanvasBackground::Dark => "Dark",
        }
    }

    /// True for backgrounds dark enough that the grid should be drawn light.
    pub fn is_dark(self) -> bool {
        matches!(self, CanvasBackground::Charcoal | CanvasBackground::Dark)
    }
}

/// How the grid is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridStyle {
    /// Faint horizontal + vertical lines.
    Lines,
    /// A dot at every grid intersection. Cleaner for dense diagrams.
    Dots,
}

/// Display unit for grid values in the ribbon. Affects labels only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GridUnits {
    Pixels,
    Mils,
    Millimeters,
    Inches,
}

impl GridUnits {
    /// Short suffix string for sliders / readouts (" px", " mm", " mil", " in").
    pub fn suffix(self) -> &'static str {
        match self {
            GridUnits::Pixels => " px",
            GridUnits::Mils => " mil",
            GridUnits::Millimeters => " mm",
            GridUnits::Inches => " in",
        }
    }

    /// Human-readable name for picker UIs.
    pub fn label(self) -> &'static str {
        match self {
            GridUnits::Pixels => "Pixels",
            GridUnits::Mils => "Mils",
            GridUnits::Millimeters => "Millimeters",
            GridUnits::Inches => "Inches",
        }
    }
}

// =============================================================================
// Node
// =============================================================================

/// A placed shape on the canvas with ports and styling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    pub kind: NodeKind,
    pub transform: Transform,
    pub overlay: Overlay,
    pub ports: Vec<Port>,
}

/// The primitive shape backing a node.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeKind {
    Rect,
    Circle,
    Ellipse,
    /// A free-form path defined by a list of segments.
    Path(Vec<PathSegment>),
    /// A node that is itself a sub-scene (hierarchical canvas).
    Group(NodeId),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

/// A connection between two endpoints.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub from: EdgeEnd,
    pub to: EdgeEnd,
    pub routing: Routing,
    pub overlay: EdgeOverlay,
}

/// One end of an [`Edge`]: anchored to a port, or a free-floating point.
///
/// The model is otherwise strictly relational — `Free` exists so that
/// deleting a wire segment can leave the surviving run intact, dangling
/// at the cut. A `Free` end is expected to be transient: dragged onto a
/// port to reconnect, or removed. The `.canvas` DSL does not yet
/// represent free ends, so a dangling wire isn't persisted on save.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeEnd {
    /// Anchored to a node's port — follows the port as the node moves.
    Port(NodeId, PortId),
    /// A free-floating world-space point.
    Free(f32, f32),
}

impl EdgeEnd {
    /// The node this end is anchored to, if it is a port end.
    pub fn node_id(&self) -> Option<&NodeId> {
        match self {
            EdgeEnd::Port(n, _) => Some(n),
            EdgeEnd::Free(..) => None,
        }
    }

    /// True iff this end is a free, dangling point.
    pub fn is_free(&self) -> bool {
        matches!(self, EdgeEnd::Free(..))
    }
}

/// Which end of an [`Edge`] — `from` or `to`. Used to address a single
/// endpoint when reattaching a dangling wire to a port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeEndSide {
    From,
    To,
}

/// How an edge is routed between its endpoints.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub enum Routing {
    #[default]
    Orthogonal,
    Bezier,
    Straight,
    /// A hand-routed wire: an orthogonal polyline threaded through these
    /// waypoints (absolute world coordinates). The full path is
    /// `[from] + waypoints + [to]`, with `from`/`to` resolved live from the
    /// ports. Dragging any segment edits the waypoints bounding it.
    Manual { waypoints: Vec<(f32, f32)> },
}

/// Visual style for an edge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Overlay {
    pub border: Border,
    pub fill: Fill,
    pub text: Option<TextLabel>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: NodeId,
    pub label: Option<String>,
    pub members: Vec<NodeId>,
}
