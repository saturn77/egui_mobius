# egui_mobius

A framework for egui applications built from **citizens** — docked
panels with stable identity — sharing state through reactive cells,
with real backends on threads. The optional `.mobius` language
describes such an app as source; `mobius_codegen` turns it into a
complete, standalone cargo project of ordinary egui Rust.

## Layout

- `crates/egui_citizen`: the citizen pattern. The `Registry` manages
  citizen lifecycle and activation — it is never the backend path.
- `crates/egui_mobius`: signal/slot bus, `Dispatcher` /
  `AsyncDispatcher` — the frontend↔backend message path.
- `crates/egui_mobius_reactive`: `Dynamic<T>` shared cells,
  `Derived<T>` — the cross-panel coupling layer.
- `crates/mobius_lang`: the `.mobius` parser, checker, and IR.
- `crates/mobius_codegen`: generator (`mobius-gen` binary) emitting
  cargo projects from `.mobius` sources.
- `examples/mobius/{terminal,dock,ribbon}`: a `.mobius` source, its
  `mobius.toml`, and the committed `generated/` project — the grammar
  reference by example.
- `book/src/mobius-lang/`: the language chapters (shared state, roles,
  signals, threaded backends).

## Build an app from a description

1. Write the app as a `.mobius` file. Learn the grammar from
   `examples/mobius/ribbon/ribbon.mobius` (top ribbon, enums, theme,
   menu block), `terminal.mobius` (docked citizens, perspectives,
   prefab shell), and the book chapters. The surface keyword for an
   interface view is `role` (`modport` is an accepted alias).
2. Generate:
   `cargo run -p mobius_codegen --bin mobius-gen -- <file>.mobius <out-dir>`
   — or put `out` in a `mobius.toml` next to the source. Parse errors
   come back with line numbers; fix the source, not the output.
3. Verify headlessly, no window, no GPU: `cargo test` inside the
   generated project boots the app and asserts every always-visible
   control reaches the accessibility tree.
   `cargo test -- --ignored --nocapture` prints the full widget tree.
4. The generated project is standalone, idiomatic egui Rust — safe to
   hand-edit from there; nothing it emits depends on the generator.

## Work on the framework itself

- Gates: `cargo test -p mobius_codegen` regenerates the three example
  apps and `cargo check --all-targets`s them (slow — minutes each,
  first run). `cargo test -p mobius_lang` covers parser/lowering.
- After changing `mobius_codegen`, regenerate the committed examples
  (`cargo run -p mobius_codegen --bin mobius-gen -- examples/mobius/<name>/<name>.mobius`)
  and commit the regenerated output with the change.
- Language work: keep `role` as the taught keyword; internals keep
  their `Modport` names. New widget emission must decide how it
  interacts with the generated headless tests (popup contents are
  hidden until clicked and must not be asserted — see
  `visible_labels` in `crates/mobius_codegen/src/lib.rs`).

## Invariants (violations are bugs)

- Panels take `&SharedState` plus an outbox — never `&mut Registry`.
- `SharedState` never crosses into backend threads; backends get
  cells and drained messages through their declared role.
- The `Registry` is lifecycle only; messages to backends go through
  outboxes and the dispatcher.
