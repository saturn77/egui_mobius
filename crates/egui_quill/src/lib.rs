//! # egui_quill
//!
//! Reactive syntax-highlighted text editor citizen for egui_mobius
//! applications. Mirrors the `egui_lens` pattern: shared state in a
//! `Dynamic<T>`, per-frame view borrowing references, atoms (the
//! editable text area, language picker, theme picker) live inside
//! the citizen panel.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use egui_mobius_reactive::Dynamic;
//! use egui_quill::{ReactiveEditor, ReactiveEditorState};
//!
//! // Create shared state once at app construction.
//! let editor_state = Dynamic::new(ReactiveEditorState::new()
//!     .with_content("// hello, world\nfn main() {}\n")
//!     .with_language("Rust"));
//!
//! // Per frame inside `ui`:
//! # let mut ui: egui::Ui = unimplemented!();
//! let editor = ReactiveEditor::new(&editor_state);
//! editor.show(&mut ui);
//! ```
//!
//! Other panels and threads observe the buffer reactively by reading
//! `editor_state.get().content`.
//!
//! ## WASM compatibility
//!
//! Fully WASM-compatible. Syntect is configured with `regex-fancy`
//! (pure Rust) instead of `regex-onig` (C bindings) so the crate
//! compiles for `wasm32-unknown-unknown`. Bundle size in release
//! wasm with `wasm_opt = "z"` is ~700KB compressed for the default
//! syntax + theme set.

mod editor;
mod state;

pub use editor::ReactiveEditor;
pub use state::{ReactiveEditorState, EDITOR_LANGUAGES, EDITOR_THEMES};
