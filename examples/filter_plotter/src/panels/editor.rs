//! Thin wrapper that adapts `egui_quill::ReactiveEditor` into the
//! tab-viewer's `panels` API. Lens did this through a sibling
//! wrapper too — see `panels/logger.rs`. Once `egui_quill` impls
//! `Citizen` directly (#30 sibling work), this wrapper can be
//! dropped and the dock layout will use the citizen view directly.

use eframe::egui;
use egui_quill::ReactiveEditor;

use crate::state::SharedState;

pub struct EditorPanel {}

impl EditorPanel {
    pub fn new() -> Self {
        Self {}
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let editor = ReactiveEditor::new(&state.editor);
        editor.show(ui);
    }
}
