# egui_grafica — GPU rendering plan

Moving `egui_grafica` from per-frame CPU tessellation to a retained
wgpu pipeline. Four phases; phases 0–3 are committed scope.

## Why

`render::paint_scene` walks every node, edge, port, waypoint, and grid
intersection on **every frame** and hands egui's tessellator a fresh pile
of shapes to re-triangulate. `GridStyle::Dots` alone emits one
`circle_filled` per grid intersection across the whole viewport. None of
that geometry changes when the user merely pans or zooms — only the
view transform does.

The fix, in one sentence: upload geometry to VRAM once, and let pan/zoom
be a uniform update instead of a re-tessellation.

This plan synthesizes a design discussion with Tim Schmidt. The load-
bearing ideas:

- Drop static geometry into the wgpu layer instead of re-tessellating it.
- Once geometry is on the GPU, pan/zoom only moves the view/clip window;
  the CPU re-uploads only when the scene graph changes.
- Instancing: 1000 copies of one component upload once, drawn N times.
- "Treat the hyperreals as truth, and VRAM as an f64 cache" — the
  hypercurve `Contour2` set in `geometry.rs` is the source of truth; the
  GPU buffers are a derived, lossy cache keyed by a generation counter.

## Ground truth (current state)

- egui 0.34 / eframe 0.34. The `grafica_quad_cluster` example runs on the
  **wgpu backend** (eframe `default` features); `egui-wgpu 0.34` is
  already resolved. No backend migration needed.
- `render.rs` was designed as the swappable backend module — its own
  docs say a wgpu backend "replaces this module wholesale."
- `geometry.rs` already bridges f32 ⇄ hypercurve `Real` and builds node
  shapes as `Contour2`. The "hyperreals as truth" substrate exists.
- Paint site: `citizen.rs::show_canvas` — `ui.allocate_painter(...)` then
  `paint_scene` / `paint_selection` / previews. GPU callbacks slot in
  here via `painter.add(egui_wgpu::Callback::new_paint_callback(...))`.

## Architecture

A new `gpu` module, behind a `gpu` cargo feature (off by default so the
crate still builds for glow / web, where `render.rs` remains the path).

- `GraficaRenderer` — owns wgpu pipelines, buffers, bind groups. Created
  once at app startup, stored in `egui_wgpu`'s `callback_resources`
  TypeMap.
- `egui_grafica::gpu::init(render_state)` — the app calls this once from
  `eframe::CreationContext`; it constructs `GraficaRenderer` and inserts
  it into `callback_resources`.
- `CanvasCallback` — implements `egui_wgpu::CallbackTrait`. Carries the
  per-frame view state. `prepare` uploads/updates buffers; `paint`
  issues draws. Added to the egui painter inside `show_canvas`.
- Text, selection halos, and rubber-band previews stay on the egui
  painter, composited on top of the callback. egui's glyph atlas is
  already GPU-cached; text-on-GPU is out of scope.

## Phase 0 — wgpu plumbing — DONE (commit, 2026-05-19)

Stand up the pipeline end to end with the smallest real payload.

- Workspace `Cargo.toml`: add `egui-wgpu`, `wgpu`, `bytemuck` to
  `[workspace.dependencies]`.
- `egui_grafica/Cargo.toml`: optional deps + `gpu` feature.
- `gpu` module: `GraficaRenderer`, `init`, `CanvasCallback`, a
  `ViewportUniform` POD struct, one fullscreen-quad pipeline.
- Deliverable: the **canvas background fill** is drawn by the GPU
  callback (a single colored fullscreen quad) instead of
  `painter.rect_filled`. Trivially verifiable, and the quad becomes the
  host for the Phase 1 grid shader.
- `grafica_quad_cluster`: enable the `gpu` feature, call `gpu::init`.

## Phase 1 — procedural grid shader — DONE (2026-05-19)

- Upgrade the fullscreen quad's fragment shader to compute the grid
  procedurally from viewport uniforms: origin, zoom, world spacing,
  line/dot style, minor/major weighting, ink color, dot size.
- Remove the per-intersection circle/line emission from `paint_grid`.
- Result: the single largest per-frame tessellation cut; grid is crisp
  at any zoom with zero geometry.

## Phase 2 — instanced scene geometry

- Phase 2a — DONE (2026-05-19): a unit-quad mesh; one GPU **instance**
  per node carrying transform, fill, border. Rect renders directly;
  circle/ellipse via an SDF in the fragment shader keyed by `NodeKind`.
  Pan/zoom updates only `ViewportUniform`.
- Phase 2b — DONE (2026-05-19): edges — each polyline segment expanded
  to an instanced, antialiased quad, with dash/dot in-shader.
  Arrowheads stay on the painter.
- Ports and waypoints stay on the egui painter — few per scene, cheap,
  constant screen size. Revisit only if profiling says otherwise.

## Phase 3 — dirty tracking — DONE (2026-05-19)

- `Registry` gained a `generation: u64` counter, bumped by every
  mutation (all funnel through `mutate` / `set_scene`).
- The GPU instance buffers record the generation they were built from.
- `CanvasCallback::prepare` compares generations and re-uploads only on
  mismatch — pan/zoom frames upload nothing but the uniform.
- This is the "VRAM as a cache" model made concrete: the cache key is
  the generation; the scene is the truth.
- Follow-up: CPU-side instance construction (`collect_edge_instances`,
  `node_instance`) still runs every frame. Deferring it behind the same
  generation key — so unchanged frames skip routing and color parsing —
  is a separable optimization, not yet done.

## Phase 4 — over-render texture cache / LOD tiling (deferred)

Tim's side-scroller technique: render visible + ~50% margin to a
texture, treat pan/zoom as a shader transform over it, re-render on
boundary cross, regenerate triangles per-axis, evict to bound VRAM.

Explicitly **not** in this pass. With instancing plus dirty tracking,
scenes of a few thousand nodes should render comfortably. Tiling is
worth building only when a profiler shows the instanced redraw is the
bottleneck — not before.

## Notes

- `gpu` feature is additive: `render.rs` stays the CPU/glow/web path.
- Keep clippy clean and the existing 34 tests green throughout.
- Commit at every phase boundary.
