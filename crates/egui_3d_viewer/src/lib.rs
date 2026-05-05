//! # egui_3d_viewer
//!
//! A minimal 3D rendering toolkit for `egui` applications, lifted
//! from CopperForge's `render3d` module. Knows nothing about gerbers,
//! CSG, or any application-specific scene type — inputs are plain
//! float vertex buffers in the `xyz rgb` stride. Callers do their
//! own scene→vertices conversion and hand down `ColoredMesh` values.
//!
//! Backend: [`glow`](https://crates.io/crates/glow) (OpenGL 3.3) via
//! `egui_glow`'s `PaintCallback`. Wasm-portable through WebGL2.
//!
//! ## Status
//!
//! This is the lower-level rendering primitives layer — `Camera`,
//! `ColoredMesh`, `UnlitProgram`, plus `axes` / `grid` helpers. The
//! reactive citizen wrapper (`ReactiveViewerState`,
//! `Reactive3dViewer`, `ViewerCitizen`) lands on top of this in a
//! follow-up.
//!
//! # Credits
//!
//! [alumina-interface](https://github.com/timschmidt/alumina-interface)
//! by Timothy Schmidt (MIT-licensed) is the direct inspiration for
//! this code. The integration pattern — single VAO+VBO meshes with
//! `xyz rgb` stride, `egui_glow::CallbackFn` wrapped in
//! `Arc<Mutex<_>>`, the `POLYGON_OFFSET_FILL` outline trick — comes
//! from alumina, and `renderer.rs` / `mesh.rs` mirror the shape of
//! alumina's `src/renderer.rs` closely.

pub mod axes;
pub mod camera;
pub mod grid;
pub mod mesh;
pub mod renderer;

pub use camera::{project, unproject_to_z0, Camera};
pub use mesh::ColoredMesh;
pub use renderer::UnlitProgram;
