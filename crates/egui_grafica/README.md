# egui_grafica

A drawable, programmable, declaratively-serializable graphics canvas — for system
block diagrams, node graphs, and everything in between.

> **Naming note.** This crate lives in the `egui_mobius` monorepo and follows the
> `egui_*` convention of its siblings (`egui_citizen`, `egui_quill`, `egui_lens`).
> The data model and `.canvas` DSL are designed to be backend-agnostic, so a
> future toolkit-independent layer (a `grafica-core` crate) can be extracted
> later as the citizen abstraction itself becomes portable.

## What it is

`egui_grafica` is a canvas citizen — a dock-panel-aware drawing surface with a
**comprehensive data model** for placing styled shapes, attaching ports, and routing
connections between them. Two use cases drive the design:

1. **System block diagrams** in the spirit of draw.io / Visio / Simulink: rectangles,
   circles, ellipses, labels, arrows. Rapid sketching of architecture, signal flow,
   and hierarchy.
2. **Node graphs** in the spirit of Blender's shader editor / TouchDesigner /
   Houdini SOPs: typed input/output sockets, bezier-routed edges, evaluation order
   implied by topology.

Both fall out of the same `Node + Port + Edge + Overlay` core. The difference is
defaults — orthogonal routing and labeled blocks for block diagrams, bezier
routing and typed sockets for node graphs.

## Why this exists

draw.io is a useful tool with a regrettable foundation. Its mxGraph XML format
mixes geometry, semantics, and style into one soup; ports are half string-keys
and half geometry; programmatic generation is brittle. `egui_grafica` separates
concerns:

| Concern    | Where it lives                                                          |
|------------|-------------------------------------------------------------------------|
| Geometry   | `Node.transform` (position, rotation, scale)                            |
| Topology   | `Edge { from: (NodeId, PortId), to: (NodeId, PortId) }`                 |
| Style      | `Overlay` (border, fill, text — fully composable, fully serializable)   |
| Layout    | Routing strategy per edge, not baked into coordinates                   |
| Source-of-truth | The `.canvas` DSL file — humans and tools can both author it       |

The data model is **the** asset. The egui renderer is one consumer of it. CLI
tools, codegen, version-control diffs, and future Masonry / web / native renderers
are equal-class consumers.

## The data model

```rust
struct Scene {
    settings: CanvasSettings,
    nodes:    Vec<Node>,
    edges:    Vec<Edge>,
    groups:   Vec<Group>,    // optional hierarchical grouping
}

struct Node {
    id:        NodeId,
    kind:      NodeKind,       // Rect | Circle | Ellipse | Path | Group
    transform: Transform,      // position, size, rotation
    overlay:   Overlay,        // visual style — see below
    ports:     Vec<Port>,
}

struct Port {
    id:     PortId,
    name:   String,
    kind:   PortKind,          // In | Out | Bidir | Untyped
    anchor: PortAnchor,        // edge-side + parametric offset
    data_type: Option<String>, // for typed node graphs (Real, Bus<32>, etc.)
}

struct Edge {
    id:      EdgeId,
    from:    (NodeId, PortId),
    to:      (NodeId, PortId),
    routing: Routing,          // Orthogonal | Bezier | Straight | Manual(segments)
    overlay: EdgeOverlay,      // color, width, style, arrowheads, label
}

struct Overlay {
    border: Border,            // color, width, line-style (solid/dashed/dotted)
    fill:   Fill,              // color, transparency, gradient (optional)
    text:   Option<TextLabel>, // value, anchor, font, style, color
}
```

**Ports are relational, not coordinate-based.** When you drag a node, every edge
attached to it follows automatically because the edge references `(NodeId, PortId)`
not `(x, y)`. This is the single most important property of the data model.

**Anchors are parametric.** `PortAnchor::East(0.5)` means halfway down the right
edge. Spreading 4 ports along an edge is `East(0.2)`, `East(0.4)`, `East(0.6)`,
`East(0.8)`. The DSL has a `spread` keyword to do this automatically.

## The `.canvas` DSL

Scenes are authored in a **first-class domain-specific language**, not in RON
or JSON. The DSL is the source of truth. The GUI is a structured editor for it.

Why a DSL and not RON:

- **Human-authored, human-diffable.** A line moved by one pixel produces one line
  of diff, not a re-serialized blob.
- **Comments and structure.** Group related shapes, annotate intent, region
  comments — all preserved through round-trips.
- **Domain vocabulary.** `wire a -> b`, `port in vin_48v @ north`, `@overlay(...)`
  reads how engineers describe systems. RON's `Edge { from: ..., to: ... }` does not.
- **Macros and reuse.** `template buck_module { ... }` and `instance buck1 : buck_module`
  let you stamp out repeated structure. JSON cannot.
- **Tool-friendly.** A parser yields an AST; the AST drives the GUI, codegen,
  linters, and diff visualizers without bespoke serializers per consumer.

### Worked example: the QuadCluster sketch

This is the hand-drawn block diagram on graph paper, transcribed:

```canvas
canvas QuadCluster {
    @settings(grid = 10, snap = true, paper = "A3", orientation = landscape)

    // --- Power section ---------------------------------------------------
    @position(120, 200) @size(120, 220)
    @overlay(
        border: solid 2 "#1F2937",
        fill:   "#DBEAFE" @ 0.90,
        text:   "BUCK\nGaN POWER\nBOARD" @ center font("Inter", 11),
    )
    node power_board : rect {
        port in    vin_48v               @ north;
        port out   ch1, ch2, ch3, ch4    @ east(spread);
        port inout fpga_intf             @ south;
    }

    // --- Sense board (quad cluster) -------------------------------------
    @position(360, 200) @size(140, 220)
    @overlay(
        border: solid 2 "#1F2937",
        fill:   "#FEF3C7" @ 0.90,
        text:   "QUAD CLUSTER\n5VDC\nSENSE BOARD" @ top_center font("Inter", 11),
    )
    node sense_board : rect {
        port in  ch1, ch2, ch3, ch4 @ west(spread);
        port out adc_bus            @ east;
    }

    // --- ADC daughter card ----------------------------------------------
    @position(580, 320) @size(120, 90)
    @overlay(
        border: solid 2 "#1F2937",
        fill:   "#FCA5A5" @ 0.90,
        text:   "ADC BOARD\nDAUGHTER" @ center font("Inter", 11),
    )
    node adc : rect {
        port in    sense_bus  @ west;
        port inout fpga_link  @ east;
    }

    // --- FPGA board ------------------------------------------------------
    @position(800, 260) @size(180, 240)
    @overlay(
        border: solid 2 "#1F2937",
        fill:   "#A7F3D0" @ 0.90,
        text:   "FPGA BOARD" @ center font("Inter", 13, bold),
    )
    node fpga : rect {
        port inout adc_link   @ west;
        port in    fpga_intf  @ south;
    }

    // --- Power → Sense (4 phase channels) -------------------------------
    @route(orthogonal) @style(color: "#2196F3", width: 2)
    wire power_board.ch1 -> sense_board.ch1;

    @route(orthogonal) @style(color: "#2196F3", width: 2)
    wire power_board.ch2 -> sense_board.ch2;

    @route(orthogonal) @style(color: "#2196F3", width: 2)
    wire power_board.ch3 -> sense_board.ch3;

    @route(orthogonal) @style(color: "#2196F3", width: 2)
    wire power_board.ch4 -> sense_board.ch4;

    // --- Sense → ADC -----------------------------------------------------
    @route(orthogonal) @style(color: "#10B981", width: 2)
    wire sense_board.adc_bus -> adc.sense_bus;

    // --- ADC ↔ FPGA ------------------------------------------------------
    @route(orthogonal) @style(color: "#EF4444", width: 2, line: solid)
    wire adc.fpga_link <-> fpga.adc_link;

    // --- FPGA → Power (long return path, manually routed) ---------------
    @route(manual segments = "v 120, h -800, v -180")
    @style(color: "#6B7280", width: 1, line: dashed)
    wire fpga.fpga_intf -> power_board.fpga_intf;
}
```

That is the entire diagram. ~80 lines of declarative DSL, every coordinate
explicit, every style explicit, fully version-controllable.

### Node-graph mode (same model, different defaults)

```canvas
canvas ShaderGraph {
    @settings(default_routing = bezier, default_node = circle)

    @position(100, 100) node tex : ellipse {
        port out rgb : color @ east;
    }

    @position(280, 100) node mul : circle {
        port in  a : color @ west(0.3);
        port in  b : color @ west(0.7);
        port out result : color @ east;
    }

    wire tex.rgb -> mul.a;
}
```

Same `Node + Port + Edge` data model. The DSL parses identically. The renderer
picks bezier routing by default because `@settings(default_routing = bezier)` is
declared.

## Crate layout

```
egui_grafica/
├── src/
│   ├── lib.rs              // re-exports
│   ├── model.rs            // Scene, Node, Edge, Port, Overlay — pure data
│   ├── lang.rs             // .canvas DSL: lexer, parser, AST, pretty-printer
│   ├── render.rs           // egui Painter rendering of the model
│   ├── interact.rs         // selection, drag, port-connect, marquee, snap
│   └── citizen.rs          // CanvasCitizen — the dock-panel integration
├── examples/               // (planned)
│   ├── quad_cluster/       //   the README's worked example
│   ├── shader_graph/       //   node-graph mode
│   └── empty_canvas/       //   the smallest possible host app
└── tests/                  // (planned)
    └── roundtrip.rs        //   every .canvas file must parse → AST → pretty → re-parse to an equivalent AST
```

Each `*.rs` file can grow into a submodule directory as it gets fleshed out
(e.g. `lang/{lexer,parser,pretty,error}.rs`); that's a mechanical refactor.

The split between `model`, `lang`, `render`, and `interact` is deliberate.
`model` has no egui dependency. `lang` has no egui dependency. Only `render`
and `citizen` import egui. A future `grafica-masonry` or `grafica-wgpu`
backend would replace exactly those two modules.

## Citizen ecosystem positioning

`egui_citizen` provides the dock-panel lifecycle abstraction. `egui_quill` is a
text-editor citizen. `egui_lens` is a logger citizen. `egui_grafica` is the
drawing-surface citizen — but unlike its siblings, the data model and DSL are
fully toolkit-agnostic, anticipating the day when "citizen" itself becomes the
portable layer and egui is one of several backends.

```
        ┌────────────────────────────────────────────────┐
        │              citizen (abstraction)             │
        │       lifecycle, identity, messaging           │
        └───┬───────────────────────────┬────────────────┘
            │                           │
   ┌────────┴────────┐         ┌────────┴───────────┐
   │  egui backend   │         │  future: masonry,  │
   │  egui_citizen   │         │  wgpu, web, ...    │
   └────────┬────────┘         └────────┬───────────┘
            │                           │
   ┌────────┴────────────────┐   ┌──────┴──────────────┐
   │ egui_quill, egui_lens   │   │ egui_grafica  (this │
   │ (egui-specific)         │   │  crate) — backend-  │
   └─────────────────────────┘   │  agnostic model+DSL │
                                 └─────────────────────┘
```

The `.canvas` DSL is the most durable artifact. Renderers will come and go;
diagrams authored today should still open in 2040.

## Design principles

- **The model owns the truth, the GUI is a structured editor for it.** Every GUI
  interaction (drag a node, recolor a fill, connect two ports) emits an edit on
  the model. The DSL serializer always reflects the current model. There is no
  hidden state in the GUI.
- **Ports are relational, edges are relational.** Geometry is a derived view, not
  a primary key. Moving a node never breaks a connection.
- **Overlay is data, not behavior.** A node's appearance is a struct you can
  read, write, and diff. No callbacks, no draw closures.
- **The DSL is reversible.** Parse `.canvas` → AST → serialize → byte-identical
  output (modulo whitespace). This is enforced by the roundtrip test suite.
- **Reactive style edits.** Each node's `Overlay` lives in a `Dynamic<Overlay>`
  (via `egui_mobius_reactive`). Edits in a property panel update the canvas
  without a global redraw and without frame-order races.
- **No XML soup. Ever.**

## Roadmap

- [ ] `model/` — Scene, Node, Port, Edge, Overlay types + Dynamic integration
- [ ] `lang/` — `.canvas` lexer (logos), parser, AST, pretty-printer
- [ ] `render/` — egui Painter implementation: shapes, text, edges (ortho/bezier/manual)
- [ ] `interact/` — selection, drag, port-to-port connect, marquee, snap-to-grid
- [ ] `citizen.rs` — `CanvasCitizen` impl: dock identity, lifecycle, undo/redo stack
- [ ] Property panel: edit `Overlay` of selected nodes reactively
- [ ] Library/palette: drag stencils onto the canvas
- [ ] Export: SVG, PNG, PDF
- [ ] Import: subset of draw.io XML (one-way)
- [ ] Typed ports + connection validation (node-graph mode)
- [ ] Hierarchical canvases (a node can BE a sub-canvas via an `@subsystem` reference)

## License

Same as the egui_mobius workspace.
