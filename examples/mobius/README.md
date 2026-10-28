# The `.mobius` examples — a tutorial ladder

These examples teach the *language*, not the crate APIs (the other
`examples/` crates do that). Each is a `.mobius` source plus its
committed `generated/` output — exactly what `mobius_codegen` produces,
browsable side by side — and each builds on the previous one.

| # | Example | Teaches | Run |
|---|---------|---------|-----|
| 1 | [`ribbon/`](ribbon) | Chrome: the top ribbon citizen — wordmark, file-group events (Path B outbox), theme picker, clock, About modal with banner image and dependency table | `cd examples/mobius/ribbon/generated && cargo run` |
| 2 | [`dock/`](dock) | Composition: the same citizen instantiated twice over two interface instances (citizen-as-module), `egui_dock` layout from `dock()` directives, and the **System citizen** — the egui_lens logger prefab with startup banner and working System Info | `cd examples/mobius/dock/generated && cargo run` |
| 3 | *terminal* (planned) | The shell citizen: a stateful prefab (input, history), commands as Path B events into the actions module | — |

Regenerate any example from its source:

```sh
cargo run -p mobius_codegen --bin mobius-gen -- examples/mobius/<name>/<name>.mobius
```

Output location and dependency roots come from each example's
`mobius.toml`. The generated projects are deliberately standalone —
own workspace, no dependency on the generator — because that's the
scaffolding contract: you own the output.
