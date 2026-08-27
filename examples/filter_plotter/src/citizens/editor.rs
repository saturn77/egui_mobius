//! Thin wrapper that adapts `egui_quill::ReactiveEditor` into the
//! tab-viewer's `citizens` API. Lens did this through a sibling
//! wrapper too — see `citizens/logger.rs`. Once `egui_quill` impls
//! `Citizen` directly (#30 sibling work), this wrapper can be
//! dropped and the dock layout will use the citizen view directly.

use eframe::egui;
use egui_quill::ReactiveEditor;

use crate::state::SharedState;

use crate::tabs::EDITOR_ID;
use egui_citizen::citizen_panel;

citizen_panel!(EditorPanel, EDITOR_ID);

impl EditorPanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let editor = ReactiveEditor::new(&state.editor);
        editor.show(ui);
    }
}
