//! # egui_3d_viewer
//!
//! Reactive 3D viewer citizen for egui_mobius applications. Mirrors
//! the `egui_lens` / `egui_quill` pattern: shared state in a
//! `Dynamic<T>`, per-frame view borrowing references, atoms — camera
//! controls, view presets, wireframe / axes / grid toggles — live
//! inside the citizen panel.
//!
//! Underneath, rendering is hand-rolled OpenGL through `egui_glow`'s
//! `PaintCallback`. The crate is wasm-portable via WebGL2.
