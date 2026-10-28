# dock — the second-citizen `.mobius` example

A complete Level-1 app shape from one `.mobius` source: a themed ribbon
on top (wordmark, 🎨 theme picker, clock), a docked two-channel panel
layout, and the **System citizen** — the egui_lens reactive logger —
along the bottom, logging the startup banner (welcome + version,
dependency table, system details) the way CopperForge does.

Two things this example proves:

- **Citizen-as-module.** The *same* citizen declarations (`Controls`,
  `Scope`) instantiated *twice*, each pair wired to its own instance of
  the *same* interface (`Channel`) — declaration vs instance, like
  instantiating a Verilog module twice with different port bindings.
  Each instance gets its own `CitizenId` and dock tab; drag the left
  slider and only the left scope moves.
- **Prefab citizens.** `LoggerState` / `LogColors` are builtin types
  backed by egui_lens; the `logger` primitive renders the reactive
  event log, and the generated host logs the startup banner into it.

Generated: the `egui_dock` layout from `@layout dock()` directives
(left split, center tabs, bottom strip), the `Tab` enum and `TabViewer`
routing, per-instance registry activation on tab click, the theme
module, and the startup banner.

```sh
cargo run -p mobius_codegen --bin mobius-gen -- examples/dock/dock.mobius
cd examples/dock/generated && cargo run
```

This example is the second codegen gate:
`crates/mobius_codegen/tests/generate_dock.rs` requires the generated
project to pass `cargo check`.

Next in the tutorial ladder: a terminal/shell citizen (example 3),
building on this one.
