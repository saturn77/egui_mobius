# `egui_3d_viewer` — the 3D viewer citizen

> **`egui_3d_viewer` is a citizen.** A docked, movable, resizable
> panel with stable identity, with hand-rolled OpenGL rendering
> through `egui_glow`'s `PaintCallback`. Atoms include the grid /
> axes toggles, the measure tool, and the standard creature
> comforts — orbit, zoom, zoom-to-region, double-click reset.

If "citizen" doesn't ring a bell yet, read [What is a
citizen?](../background/what_is_a_citizen.md) first.

## What it does

`egui_3d_viewer` is the canonical 3D viewport for `egui_mobius`
applications. It renders consumer-supplied triangle and line meshes
plus a default XYZ axes gizmo and ground grid, with mouse-driven
orbit / zoom / pan and a measure tool on the Z=0 plane.

The crate sits at `crates/egui_3d_viewer/` as a sibling of
`egui_lens` and `egui_quill`. It launched in `egui_mobius` v0.4.0
extracted from CopperForge's `render3d` module. Backend is
[`glow`](https://crates.io/crates/glow), the same low-level GL
binding the rest of the eframe + egui_glow stack uses; the crate is
wasm-portable through WebGL2.

## The shape — divergent from lens / quill

Lens and quill follow a `(state, view)` split: state in a
`Dynamic<T>`, a per-frame view that borrows the state. The 3D
viewer can't follow that pattern cleanly. Persistent GL handles,
the orbit camera, and in-flight drag state all belong on a struct
that lives across frames, none of which fit cleanly inside a
reactive cell.

So `ViewerCitizen` owns everything:

- **`ReactiveViewerState`** — atom UI state: `show_grid`,
  `show_axes`, `measure_active`, `background_color`, plus a
  `reset_view_requested` command flag. Held in
  `Dynamic<ReactiveViewerState>` for cross-panel observability.
- **`ViewerCitizen`** — the citizen struct itself. Carries the
  reactive state cell, the `Camera`, lazily-initialised
  `GpuResources`, in-flight drag state, and the `CitizenState`
  handle. Its `show(ui, gl)` method is the per-frame render call.

```rust,ignore
use egui_3d_viewer::ViewerCitizen;
use egui_citizen::{CitizenId, Dispatcher};

// At app construction
let mut dispatcher = Dispatcher::new();
let viewer_state = dispatcher.register(CitizenId::new("viewer"));
let mut viewer = ViewerCitizen::new("viewer", viewer_state);

// Per frame inside `ui()` — pass the glow context from eframe::Frame
viewer.show(ui, frame.gl());
```

That's the integration. Other panels read atom state via
`viewer.state().get().show_grid` and similar; the viewer reads its
own state each frame and acts on the toggles.

## The atoms

The viewer's atoms — the user-facing controls inside the panel:

| Atom | Trigger | What it does |
|---|---|---|
| Orbit | Left-drag on canvas | Yaw + pitch the camera around the scene |
| Zoom | Scroll wheel (canvas-hovered) | Multiplicative camera distance |
| Zoom-to-region | Right-drag, release | Frame the dragged box; un-projects to Z=0 |
| Reset view | Double-click | Snap back to the default tilted top-down |
| Toggle grid | `G` key (canvas-hovered) | Flip `state.show_grid` |
| Measure | `M` key (canvas-hovered) | Flip `state.measure_active`; left-drag draws a Z=0 distance line |
| Toggle axes | Set `state.show_axes` | Hide / show the axes gizmo + screen labels |

Hover-gating on `G` and `M` matters — typing those keys in another
panel must not flip the viewer's settings under the user. The
zoom-to-region overlay and the measure-tool line are painted with
egui's 2D painter on top of the GL pass so they stay visible
regardless of camera angle.

## Scene injection

The default scene is the axes gizmo + a ground grid; consumer apps
push their own meshes through:

- `viewer.set_scene_triangles(verts)` — flat `xyz rgb` buffer, six
  floats per vertex, drawn with the `TRIANGLES` primitive.
- `viewer.set_scene_lines(verts)` — same stride, drawn as `LINES`.
  Useful for wireframe overlays or vector-style content.
- `viewer.clear_scene()` — drop both back to the empty default.

Uploads are deferred to the next `show()` call — a glow context is
only available there. The buffer format is intentionally minimal:
the citizen knows nothing about the consumer's domain. Build your
scene however makes sense — CSG with `csgrs`, gerber polygon
extrusion, hand-built meshes — then convert to the float-buffer
shape and hand it over.

```rust,ignore
let plate = build_plate_with_holes();         // your scene
let verts = mesh_to_xyz_rgb(&plate, color);   // your conversion
viewer.set_scene_triangles(verts);
viewer.set_axes_length(scene_max_dim * 0.15);
viewer.camera_mut().fit_to_bbox(scene_w, scene_h);
```

## WASM

The viewer is wasm-portable through WebGL2. `egui_glow` ships a
WebGL2 backend out of the box, and the underlying renderer code
uses only OpenGL 3.3 / WebGL2-safe features — no compute shaders,
no extension-gated texture formats. The same crate compiles for
`wasm32-unknown-unknown` with `default-features = false` on
`egui_glow`.

## Performance

GPU resources are lazily initialised on the first frame where a
glow context is in scope, then cached on the citizen. The shader
program, axes mesh, grid mesh, and the two scene-mesh slots are
allocated once. Re-uploads happen only when the consumer calls a
`set_scene_*` method — the pending vec is drained inside `show()`,
not on every frame.

Per-frame cost on an empty default scene is two line-draw calls
plus the egui paint pass. Per-frame cost on a populated scene adds
one or two more draw calls — depth-tested geometry first, then
grid + lines + axes layered on top.

## Backend stack — glow now, wgpu later

The viewer is built on hand-rolled OpenGL 3.3 through `egui_glow`'s
`PaintCallback`. The shader/mesh shape — single VAO+VBO meshes with
`xyz rgb` stride, the `Arc<Mutex<_>>` callback wrapper — comes
directly from
[Tim Schmidt's alumina-interface](https://github.com/timschmidt/alumina-interface),
which is the reference implementation for this integration pattern.

A future migration to `wgpu` is on the roadmap once the API is
stable. The migration is mechanical: the public surface —
`ViewerCitizen`, `ReactiveViewerState`, `set_scene_*` — does not
need to change; only the underlying `UnlitProgram` / `ColoredMesh`
implementations port to wgpu's pipeline / buffer model.

## See also

- `examples/viewer3d_csgrs` — full working example. Builds a
  6"×4"×1/8" PCB mounting plate with six mounting holes via
  `csgrs` constructive solid geometry, hands the triangulated
  mesh to the viewer, demonstrates every creature comfort.
- [`egui_lens`](lens.md) — sibling citizen for logging.
- [`egui_quill`](quill.md) — sibling citizen for text editing.
- [`alumina-interface`](https://github.com/timschmidt/alumina-interface)
  — the reference implementation for `egui_glow` + glow + nalgebra
  3D rendering inside an egui app.

---

*Chapter last revised: 2026-05-05 — egui_mobius v0.4.0.*
