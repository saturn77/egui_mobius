# The `.mobius` examples — a tutorial ladder

These examples teach the *language*, not the crate APIs (the other
`examples/` crates do that). Each is a `.mobius` source plus its
committed `generated/` output — exactly what `mobius_codegen` produces,
browsable side by side — and each builds on the previous one.

| # | Example | Teaches | Run |
|---|---------|---------|-----|
| 1 | [`ribbon/`](ribbon) | Chrome: the top ribbon citizen — wordmark, file-group events (Path B outbox), theme picker, clock, About modal with banner image and dependency table | `cd examples/mobius/ribbon/generated && cargo run` |
| 2 | [`dock/`](dock) | Composition: the same citizen instantiated twice over two interface instances (citizen-as-module), `egui_dock` layout from `dock()` directives, and the **System citizen** — the egui_lens logger prefab with startup banner and working System Info | `cd examples/mobius/dock/generated && cargo run` |
| 3 | [`terminal/`](terminal) | The Workbench: a nearly full ribbon (file group firing Path B events routed into the logger), plus the **terminal prefab** — a *port-less* shell citizen (all state citizen-local) with colour-coded scrollback, built-ins (`help`/`ver`/`info`/`clear`) and `bash -c` passthrough, CopperTrace-style | `cd examples/mobius/terminal/generated && cargo run` |
| 4 | [`plotter/`](plotter) | The **plot block** — the EDA waveform view as declared source: a `Vec<f32>` trace cell rendered through `egui_plot`, with `scale --` (vertical gain, the amplitude slider) and `show --` (channel select, the enable checkbox) consumed live by the view. SignalBench's shape on the generated path | `cd examples/mobius/plotter/generated && cargo run` |

## From description to source

Each example realizes a brief a user might actually give — the pairing
an agent (or a newcomer) learns the language from. The smallest
possible pair, in full:

> *"A control strip: an enable checkbox, a gain slider from 0 to 10,
> and a Ping button that fires an event to the backend."*

```mobius
app ControlStrip {
    enum Cmd { Ping }

    interface Panel {
        enabled : bool = false
        gain    : f32 = 1.0
        events  : signal Cmd
        role ui   (write enabled, write gain, emit events)
        role host (read enabled, read gain, drain events)
    }

    citizen Controls (p : Panel.ui) {
        row {
            checkbox "Enabled" -- p.enabled;
            slider "Gain" 0.0..10.0 -- p.gain;
            button "Ping" -> p.events(Cmd::Ping);
        }
    }

    @wiring {
        let panel = Panel();
        let controls = Controls(p = panel.ui);
    }

    @layout {
        top(controls);
    }
}
```

Every phrase of the brief lands somewhere declared: the two cells are
interface fields, the button is a `signal` send, and the two `role`s
say which side may write, read, emit, and drain. Generate it, and
`cargo test` in the output proves the three controls reached the
accessibility tree — headless, no GPU.

The ladder's briefs, in the same spirit:

- **ribbon** — *"A top command bar: wordmark on the left; New, Open,
  Save; Build and Emit actions; a Layout dropdown; on the right a
  theme picker, an About button, and a live clock. About opens a
  modal with our banner image and a dependency table."* →
  [`ribbon/ribbon.mobius`](ribbon/ribbon.mobius), ~65 lines.
- **dock** — *"Two identical channel panels — controls plus scope —
  docked side by side, each with its own state, and a system logger
  below."* The same citizen declared once, instanced twice over two
  interface instances → [`dock/dock.mobius`](dock/dock.mobius).
- **terminal** — *"A workbench: the ribbon fires file events into a
  logger, a shell panel runs built-ins and bash one-liners, and the
  docked layout saves named perspectives with a startup default."* →
  [`terminal/terminal.mobius`](terminal/terminal.mobius).
- **plotter** — *"Bench controls — an amplitude slider and an enable
  switch — and a scope plotting the trace."* →
  [`plotter/plotter.mobius`](plotter/plotter.mobius), the plot block
  on the generated path. The trace ships as a static waveform
  default; a *live* trace needs a writer on a thread, which is the
  handler milestone. The full live shape — worker thread draining
  commands, writing the trace a plot observes — already runs on the
  *live elaboration* path:
  [`../mobius_lang_demo/app.mobius`](../mobius_lang_demo/app.mobius)
  (`cargo run -p mobius_lang_demo` — edit the `.mobius` while it runs
  and it re-elaborates on save).

Regenerate any example from its source:

```sh
cargo run -p mobius_codegen --bin mobius-gen -- examples/mobius/<name>/<name>.mobius
```

Output location and dependency roots come from each example's
`mobius.toml`. The generated projects are deliberately standalone —
own workspace, no dependency on the generator — because that's the
scaffolding contract: you own the output.
