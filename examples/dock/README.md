# dock — the second-citizen `.mobius` example

The honest test of citizen-as-module: the **same** citizen declarations
(`Controls`, `Scope`) instantiated **twice**, each pair wired to its own
instance of the **same** interface (`Channel`) — declaration vs
instance, like instantiating a Verilog module twice with different port
bindings. Each instance gets its own `CitizenId` (via the
`citizen_panel!` `with_id` constructor) and its own dock tab; drag the
left slider and only the left scope moves.

Generates: `egui_dock` layout from the `@layout dock()` directives
(left split + center tabs), a `Tab` enum, the `TabViewer` routing each
tab to its instance with its own state bindings, and per-instance
registry activation on tab click.

```sh
cargo run -p mobius_codegen --bin mobius-gen -- examples/dock/dock.mobius
cd target/tmp/dock_demo && cargo run
```

This example is the second codegen gate:
`crates/mobius_codegen/tests/generate_dock.rs` requires the generated
project to pass `cargo check`.
