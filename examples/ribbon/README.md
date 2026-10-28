# ribbon — the first `.mobius` → Rust example

A top command bar in the shape of CopperTrace's hand-written
`top_ribbon.rs`, declared as ~57 lines of `.mobius` source. Generates a
complete standalone cargo project: themed ribbon citizen (Path B outbox
for its button events), `Dynamic<T>` state cells (Path A), registry
lifecycle wiring, theme palettes, and a resizable About window with
banner image and dependency table.

```sh
cargo run -p mobius_codegen --bin mobius-gen -- examples/ribbon/ribbon.mobius
cd examples/ribbon/generated && cargo run
```

The output location and dependency roots come from `mobius.toml` here;
CLI arguments override it. This example doubles as the codegen gate:
`crates/mobius_codegen/tests/generate_ribbon.rs` parses it, emits the
project, and requires `cargo check` to pass — if this example breaks,
the build does.
