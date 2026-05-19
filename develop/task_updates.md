# egui_grafica — updates log

Chronological log of `egui_grafica` changes, newest first. Spans
2026-05-14 → 2026-05-19.

## GPU rendering — retained wgpu pipeline (`gpu` feature)

See `develop/gpu_rendering_plan.md` for the staged plan.

- Phase 2a — instanced node bodies on the GPU: one instance per node,
  rect / circle / ellipse via a fragment-shader SDF, inside border
  stroke. Edges, text, ports, waypoints, selection stay on the painter.
- Phase 1 — procedural grid shader: the grid is computed per-pixel from
  the viewport transform, replacing thousands of tessellated circles
  and line segments with zero geometry.
- Phase 0 — wgpu plumbing: `GraficaRenderer` resource, `gpu::init`,
  `CanvasCallback`; the canvas background became a fullscreen GPU quad.

## Interaction & geometry

- `7aef258` — gate canvas gestures to the primary mouse button so a
  right-click never starts a pan or drag.
- `0e97026` — marquee selection, middle-button pan, wrapping ribbon.
- `389908f` — drag ports along the node perimeter (spatial `Connecting`
  gesture, latches into connection-draw once the cursor leaves the node);
  double-click a pivot to delete it.
- `dade252` — selectable canvas background (Light / Slate / Charcoal /
  Dark); grid ink flips to stay visible on dark backgrounds.
- `a58f434` — node shapes as hypercurve `Contour2`; exact point-in-contour
  hit-testing replaces the bounding-box approximation.
- `a8b200b` — bezier routing through hypercurve's certified flattener
  (provable flatness bound).
- `aecb6cf` — adopted hypercurve as the geometry kernel; added the
  `geometry` f32 ⇄ `Real` bridge.
- `4d907e3` — waypoint-based wire re-routing: drag any segment (both axes),
  double-click a wire to insert a pivot vertex.
- `1ce2f02` — first wire re-routing pass (`DraggingSegment` gesture).
- `8fb653c` — canvas interaction state machine (`CanvasFsm`).
- `31350f6` — fix: hit-test at the true press origin so thin wires and
  small ports select reliably instead of falling through to panning.
- `c9e8e9b` — extracted connection routing into a dedicated `router`
  module.

## Connections & editing

- `ae76f61` — edit wire colour, width, and line style from the context
  menu (a 'Wire style' submenu); edits flow through
  `Registry::update_edge_overlay`.
- `589fb0d` — right-click context menu: delete pivot/segment/wire, add a
  connection port to a node.
- `e134f31` — select and delete wires (and nodes); `Delete`/`Backspace`.
- `d6ec427` — create connections by dragging port → port.
- `2b14bb4` — node selection and drag (shift multi-select, snap-to-grid).

## DSL

- `31baf5e` — load/save `.canvas` files from the ribbon File menu.
- `b002a7c` — comment preservation through a `.canvas` round-trip.
- `76e22d3` — implemented the `.canvas` DSL: lexer, parser, pretty-printer.

## Foundations

- `d21d39a` — render pipeline, `Registry` backend-model, dockable ribbon.
- `9167828` — new crate: programmable canvas citizen, DSL-first model.

## Notes

- 34 unit tests, clippy clean throughout.
- Local-only commits may be ahead of origin — check `git status` before
  assuming the remote is current.
