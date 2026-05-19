# egui_grafica — task progress

Status snapshot of the `egui_grafica` crate (a programmable canvas citizen:
system block diagrams + node graphs, hybrid of schematic-entry and Simulink,
with a round-trip `.canvas` DSL).

Last updated: 2026-05-19.

## Working

**Data model & registry**
- `model.rs` — pure data: `Scene`, `Node`, `Port`, `Edge`, `Overlay`,
  `CanvasSettings`. No egui dependency. serde-derived.
- `registry.rs` — `Registry` wraps `Dynamic<Scene>`; the single typed
  mutation surface. Every scene edit goes through it.

**DSL (`lang.rs`)**
- Keyword-block `.canvas` syntax: hand-written lexer, recursive-descent
  parser, canonical pretty-printer.
- Round-trip stable: `parse(&pretty(&s))` reconstructs an equal `Scene`.
- Comment preservation for top-level items via `parse_document` /
  `pretty_document`.

**Routing (`router.rs`)**
- Dedicated module — the single source of routing truth.
- Orthogonal, straight, bezier, and manual (waypoint) routing.
- Bezier routes flattened through hypercurve's certified flattener.

**Geometry kernel (`geometry.rs`)**
- hypercurve is the geometry foundation. `geometry.rs` is the f32 ⇄ `Real`
  bridge — f32→Real is lossless, Real→f32 is the only lossy step.
- Node shapes build as hypercurve `Contour2`; hit-testing uses exact
  contour containment.

**Rendering (`render.rs`)** — CPU path / glow / web backend
- Viewport (pan/zoom), grid (lines/dots), nodes (rect/circle/ellipse),
  edges, ports, waypoint handles, selection highlights.
- Selectable canvas background (Light / Slate / Charcoal / Dark).

**GPU rendering (`gpu` module, `gpu` cargo feature)** — wgpu path
- Retained wgpu pipeline; pan / zoom is a uniform update, not a
  re-tessellation. See `develop/gpu_rendering_plan.md`.
- Phase 0 — wgpu plumbing: `GraficaRenderer` resource, `gpu::init`,
  `CanvasCallback`. Done.
- Phase 1 — procedural grid shader: grid computed per-pixel, zero
  geometry. Done.
- Phase 2a — instanced node bodies: one instance per node, rect /
  circle / ellipse via fragment SDF, inside border stroke. Done.
- Phase 2b (edges) and Phase 3 (dirty tracking) — pending.
- Off by default; the example `grafica_quad_cluster` enables it.

**Interaction (`interact.rs` + `citizen.rs`)**
- `CanvasFsm` — explicit interaction state machine: Idle, Panning,
  MovingNodes, Connecting, DraggingSegment, DraggingWaypoint.
- Node selection + drag (shift multi-select), snap-to-grid.
- Connection creation: drag port → port.
- Port repositioning: drag a port along its node's perimeter.
- Wire selection + deletion; segment re-routing (both axes); pivot
  insertion (double-click wire) and deletion (double-click pivot).
- Dockable ribbon: grid/units/routing/background controls, hotkeys
  (G/X/Y/R, Delete), File menu (load/save `.canvas`).

**Demo** — `examples/grafica_quad_cluster/`.

## Stubbed / not yet done

- `NodeKind::Path` (freeform shapes) and `NodeKind::Group` (sub-canvases)
  render as rectangles; not editable.
- Obstacle-aware routing (wire-around-node) — the `Contour2` / curve
  representation needed for it is in place; the routing logic is not.
- A live two-way DSL pane (text edits ↔ canvas) — DSL syncs at load/save
  only.
- Net-equivalence connection model (undirected nets joined by labels);
  edges are currently point-to-point and directed.
- Wire segment selection when a wire has no intermediate waypoints —
  cannot yet click/select/delete a single orthogonal run.
- Movable left-column shape ribbon with CAD-style tool icons (rect,
  square, circle, ellipse, parallelogram, text).

## Tests

34 unit tests, clippy clean.
