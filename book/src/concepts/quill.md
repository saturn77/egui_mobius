# `egui_quill` — the syntax-highlighted editor

> **`egui_quill` is a citizen.** A docked, movable, resizable panel
> with stable identity (`"editor"`) and a known set of atoms inside —
> the editable monospace text area, the language picker, the theme
> picker. Like every other citizen panel, it observes shared
> reactive state through `Dynamic<T>` and participates in
> dispatcher-coordinated activation.

If "citizen" doesn't ring a bell yet, read [What is a
citizen?](../background/what_is_a_citizen.md) first.

## What it does

`egui_quill` is the canonical text editor for the `egui_mobius`
ecosystem. It provides a syntax-highlighted, monospace editing
panel with language and theme pickers as in-panel controls. The
buffer lives in a `Dynamic<ReactiveEditorState>`, so every keystroke
is observable from any other panel or backend thread.

Out of the box: Rust, JSON, YAML, Python, JavaScript, Markdown,
Plain Text. Themes from syntect's default set (base16-ocean.dark,
Solarized, etc.).

Quill lives in `crates/egui_quill/` as a sibling of
`egui_lens`. It launched in `egui_mobius` v0.4.0 alongside the
broader canonical-citizen-panels initiative.

> *Implementation note:* like lens, quill currently ships as a
> widget that consuming apps wrap in a thin panel struct. Once a
> sibling `EditorCitizen` lands — parallel to lens's path — the
> wrapper goes away and the dock layout uses the citizen view
> directly.

## The shape

Two types matter: state and view.

- **`ReactiveEditorState`** is the data — text buffer, active
  language name, active theme name. Held in
  `Dynamic<ReactiveEditorState>` for cross-panel observability.
- **`ReactiveEditor<'a>`** is the view — a per-frame widget
  borrowing the state. Construct inside `ui()`, render via
  `.show(ui)`.

```rust,ignore
use egui_mobius_reactive::Dynamic;
use egui_quill::{ReactiveEditor, ReactiveEditorState};

// At app construction
let editor_state = Dynamic::new(
    ReactiveEditorState::new()
        .with_content("// hello, world\nfn main() {}\n")
        .with_language("Rust")
        .with_theme("base16-ocean.dark"),
);

// Per frame inside `ui()`
let editor = ReactiveEditor::new(&editor_state);
editor.show(ui);
```

That's the entire integration. Other panels and backend threads
see edits by reading `editor_state.get().content` — same reactive
pattern as lens, no callbacks, no wiring.

## The atoms

Quill's atoms are the user-visible widgets inside the panel:

| Atom | Type | What it controls |
|---|---|---|
| Language picker | `ComboBox` | Active syntax (Rust, JSON, YAML, etc.) |
| Theme picker | `ComboBox` | Color theme (base16, Solarized, …) |
| Text area | `TextEdit::multiline` | The buffer content; emits edits to `editor_state` |

Each atom corresponds to a field on `ReactiveEditorState`. Setting
the field via the picker is reactive: a backend thread observing
`editor_state` sees the language change immediately and could, for
example, trigger a parser run.

## Performance

Two layers of caching keep frame cost small:

1. **`SyntaxSet` and `ThemeSet` are `thread_local`** — loaded once
   per thread, shared across all editor instances. Cold-load is ~50
   ms; warm reads are zero-cost.
2. **The `LayoutJob` is cached** keyed on
   `(content_hash, language, theme)`. Egui calls the layouter every
   frame; with no edits and no picker changes, the cache returns the
   prior `LayoutJob` directly — no re-highlight cost.

For the typical editor session (occasional keystrokes, mostly
read-only) the per-frame cost is dominated by egui's normal text
layout, not syntax highlighting.

## WASM

Quill is fully WASM-compatible. The `syntect` crate is configured
with `regex-fancy` (pure Rust) instead of `regex-onig` (C
bindings), so the crate compiles for `wasm32-unknown-unknown`.
Bundle size impact in release wasm with `wasm_opt = "z"` is
~700KB additional for the syntect grammars and themes.

The `filter_plotter` example demonstrates this — its WASM build
includes a working quill panel as one of its dock tabs, and the
Pages-deployed demo is a click-and-edit reactive editor in the
browser.

## Custom languages — beyond the defaults

Out of the box, quill exposes whatever languages syntect's default
syntax set ships. For domain-specific languages — including HDL
(Verilog / SystemVerilog / VHDL) for engineering apps — the
roadmap is hand-rolled parser crates following the
`lexer / parser / ast / error / highlight` shape used elsewhere in
the saturn77 ecosystem (see `RustQt/venerate/svx_parser/` and
`simcore/simcore-lang/`).

Each parser crate exposes a `Highlighter` trait impl that quill
consumes; `ReactiveEditorState` carries an
`Option<Arc<dyn Highlighter>>` to enable language-specific
highlighting beyond the defaults.

## Wiring quill into a citizen panel (current pattern)

Until `EditorCitizen` lands, the consuming app provides a thin
wrapper panel struct:

```rust,ignore
use egui_quill::ReactiveEditor;
use crate::state::SharedState;

pub struct EditorPanel {}

impl EditorPanel {
    pub fn new() -> Self { Self {} }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let editor = ReactiveEditor::new(&state.editor);
        editor.show(ui);
    }
}
```

`SharedState` carries the `Dynamic<ReactiveEditorState>` field;
the `egui_dock` `TabViewer` calls `panel.show(ui, state)` for the
editor tab.

## See also

- `examples/filter_plotter` — full working example. Quill is the
  fourth dock tab grouped with Plot. Default content is the
  example's own `backend/iir.rs` so the editor renders real Rust
  syntax highlighting on first paint.
- [`egui_lens`](lens.md) — sibling citizen for logging. Same
  state/view shape; quill's API was designed to mirror lens for
  consistency.

---

*Chapter last revised: 2026-05-04 — egui_mobius v0.4.0.*
