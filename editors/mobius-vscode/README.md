# mobius-lang for VS Code

Syntax highlighting for `.mobius` composition files (the `mobius_lang`
front end of egui_mobius).

TextMate grammar only — no language server. Keywords, modport verbs,
binding arrows (`<->` / `<-` / `->`), `@wiring` / `@layout` / `@style`
sections, widget primitives, enum variant paths, strings, numbers, and
`//` comments.

## Install (local, no packaging)

Symlink the directory into VS Code's extension dir and reload:

```sh
ln -s "$(pwd)/editors/mobius-vscode" ~/.vscode/extensions/atlantix-eda.mobius-lang-0.1.0
```

Then run **Developer: Reload Window**. `.mobius` files pick up the
`mobius` language automatically.

Widget names are an *open* set in the parser — the grammar lists the
common ones; unknown primitives still parse fine, they just render as
plain identifiers until added here.
