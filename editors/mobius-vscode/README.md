# mobius-lang for VS Code

Syntax highlighting and live parse diagnostics for `.mobius`
composition files (the `mobius_lang` front end of egui_mobius).

Two layers:

- **TextMate grammar** (declarative): keywords, modport verbs, binding
  arrows (`<->` / `<-` / `->`), `@wiring` / `@layout` / `@style`
  sections, widget primitives, enum variant paths, strings, numbers,
  `//` comments.
- **Language client** → `mobius-lsp`: red squiggles from
  `mobius_lang::parse` errors as you type. Tier 1 (parse) only;
  `lower()` semantics wait on the open/closed vocabulary
  reconciliation (see `crates/mobius_codegen/ROADMAP.md`).

## Install (local, no packaging)

```sh
# the server
cargo install --path crates/mobius_lsp

# the extension (symlink + client dependency)
cd editors/mobius-vscode && npm install
ln -s "$(pwd)" ~/.vscode/extensions/atlantix-eda.mobius-lang-0.2.0
```

Flatpak VS Code keeps extensions in
`~/.var/app/com.visualstudio.code/data/vscode/extensions/` instead,
and newly dropped folders must be added to its `extensions.json`
registry. Host-built server binaries spawn fine inside the sandbox.

Then **Developer: Reload Window**. The server path resolves as:
`mobius.lsp.path` setting → `~/.cargo/bin/mobius-lsp` → `mobius-lsp`
on PATH.

Widget names are an *open* set in the parser — the grammar lists the
common ones; unknown primitives still parse fine, they just render as
plain identifiers until added here.
