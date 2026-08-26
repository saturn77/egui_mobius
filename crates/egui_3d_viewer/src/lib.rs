//! # egui_3d_viewer
//!
//! Reactive 3D viewer citizen for egui_mobius applications. Built
//! on hand-rolled OpenGL through `egui_glow`'s `PaintCallback`;
//! wasm-portable via WebGL2.
//!
//! ## Layers
//!
//! - **Render primitives** — `Camera`, `ColoredMesh`, `UnlitProgram`
//!   plus `axes` / `grid` vertex helpers. Self-contained, scene-
//!   agnostic; lifted from CopperForge's `render3d/` module.
//! - **Citizen wrapper** — `ViewerCitizen` + `ReactiveViewerState`.
//!   Owns the camera + GL handles + input drag state, exposes a
//!   `Dynamic<ReactiveViewerState>` for cross-panel observability of
//!   atom toggles, and integrates with `egui_citizen`.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use egui_citizen::{CitizenId, Registry};
//! use egui_3d_viewer::ViewerCitizen;
//!
//! # let mut registry = Registry::new();
//! // At app construction:
//! let viewer_state = registry.add().with_name("viewer");
//! let mut viewer = ViewerCitizen::new("viewer", viewer_state);
//!
//! // Per frame inside the dock TabViewer for the viewer tab:
//! # let mut ui: egui::Ui = unimplemented!();
//! # let gl: Option<&std::sync::Arc<glow::Context>> = None;
//! viewer.show(&mut ui, gl);
//! ```
//!
//! Other panels and threads observe atom toggles by reading
//! `viewer.state().get().show_grid`, etc.
//!
//! ## API shape — slight divergence from `egui_lens` / `egui_quill`
//!
//! Lens and quill use a `(state, view)` split — state in a
//! `Dynamic<T>`, a per-frame view that borrows the state. The 3D
//! viewer can't follow that pattern cleanly: persistent GL handles,
//! the orbit camera, and in-flight drag state belong on the citizen
//! itself, not inside a reactive cell. So `ViewerCitizen` owns
//! everything; the reactive `Dynamic<ReactiveViewerState>` cell is
//! just for atom UI state that other panels want to observe.
//!
//! ## Scene injection
//!
//! Default scene is an XYZ axes gizmo + a 10×10 ground grid at Z=0.
//! Consumer apps push their own meshes through:
//!
//! - `viewer.set_scene_triangles(verts)` — flat `xyz rgb` buffer,
//!   six floats per vertex, drawn with the `TRIANGLES` primitive.
//! - `viewer.set_scene_lines(verts)` — same stride, drawn as `LINES`.
//!   Useful for wireframe overlays or vector-style content.
//! - `viewer.clear_scene()` — drop both back to the empty default.
//!
//! Uploads are deferred to the next `show()` call — a glow context
//! is only available there.
//!
//! # Credits
//!
//! [alumina-interface](https://github.com/timschmidt/alumina-interface)
//! by Timothy Schmidt (MIT-licensed) is the direct inspiration for
//! the render primitives layer. The integration pattern — single
//! VAO+VBO meshes with `xyz rgb` stride, `egui_glow::CallbackFn`
//! wrapped in `Arc<Mutex<_>>`, the `POLYGON_OFFSET_FILL` outline
//! trick — comes from alumina, and `renderer.rs` / `mesh.rs` mirror
//! the shape of alumina's `src/renderer.rs` closely.

pub mod axes;
pub mod camera;
pub mod citizen;
pub mod grid;
pub mod mesh;
pub mod renderer;
pub mod state;

pub use camera::{project, unproject_to_z0, Camera};
pub use citizen::ViewerCitizen;
pub use mesh::ColoredMesh;
pub use renderer::UnlitProgram;
pub use state::ReactiveViewerState;
