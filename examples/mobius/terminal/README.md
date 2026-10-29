# terminal — example 3: the Workbench

Example 2 grown into a working tool shape: a **nearly full ribbon**
(file group firing `RibbonCmd` events — routed through actions into the
logger — plus theme, About, clock), the docked channels, the System
logger, and the **terminal prefab**: a shell citizen in the shape of
CopperTrace's `citizens/terminal.rs` — colour-coded scrollback, a `$`
prompt, built-ins (`help`, `ver`, `info`, `clear`) and `bash -c`
passthrough. Logger and Shell share the bottom strip as tabs, the
ForgePM arrangement.

Two firsts in the ladder:

- **A port-less citizen.** `citizen Shell () { terminal; }` — the
  terminal's state (scrollback, input buffer) is entirely
  citizen-local, so it declares no interface at all. Prefabs own what
  nobody else needs.
- **Actions that land somewhere.** The ribbon's Path B events no
  longer stub to stderr: with a logger wired, the generated actions
  module logs each command — click Build, watch the Logger.

Try in the terminal: `ver` (the dependency pedigree), `info` (host
details), `ls -la` (bash passthrough), `clear`.

```sh
cargo run -p mobius_codegen --bin mobius-gen -- examples/mobius/terminal/terminal.mobius
cd examples/mobius/terminal/generated && cargo run
```

Third codegen gate: `crates/mobius_codegen/tests/generate_terminal.rs`.
