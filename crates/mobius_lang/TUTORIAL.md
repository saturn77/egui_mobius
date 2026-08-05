# mobius_lang in five minutes

`mobius_lang` describes an egui_mobius app as text and elaborates it live.
This is the whole path from nothing to a running window.

## 1. Run the demo

```shell
cargo run -p mobius_lang_demo
```

Three dockable citizen tabs: `controls` (built from widget primitives in the
file), `plot` (real `egui_plot`), `log` (real `egui_lens`). Move the slider,
press Apply, watch the plot and log react. Now edit the file below and save —
the app re-elaborates while it runs.

## 2. The smallest app

A `.mobius` file has one `app`. Inside: shared state, the citizens that touch
it, and where they sit.

```rust
app Hello {
    // shared reactive state — one Dynamic<T> each
    interface State {
        level : f32 = 0.5
        // a modport is one party's view: what it may write (out) / read (in)
        modport ui (out level)
    }

    // a citizen built from widget primitives — no Rust, pure source
    citizen Panel (s : State.ui) {
        column {
            slider "Level" 0.0..1.0 <-> s.level;   // <-> edits an `out` value
        }
    }

    @wiring {
        let s     = State();          // mint the shared values
        let panel = Panel(s = s.ui);  // hand the citizen its view
    }

    @layout {
        dock(panel, region = center);
    }
}
```

That is a complete, runnable app: one slider bound to one reactive value.

## 3. The three moving parts

- **Shared state** lives in an `interface` as `Dynamic<T>` fields. A `modport`
  gives each citizen a directional view — `out` (writes), `in` (observes),
  and for `signal` fields `emit` (pushes onto a `Signal`) / `drain` (a `Slot`
  run on its own thread — the backend). Exactly one writer per value is
  enforced when you elaborate.
- **Citizens** are either *source* (built from primitives: `checkbox`,
  `slider`, `text`, `button`, `label`, in `column`/`row`/`group`) or *plugins*
  that wrap a real crate (`PlotPanel`, `LensLogger`, …).
- **Sections**: `@wiring` instantiates and connects; `@layout` docks by region
  (`center`, `left`, `right`, `above`, `below`, with an optional `fraction`).

## 4. Bindings, at a glance

| In source            | Means                                    |
|----------------------|------------------------------------------|
| `<-> path`           | two-way: widget edits an `out` value       |
| `<- path`            | read-only: widget shows an `in` value      |
| `-> sig.send(Event)` | push onto an `emit` signal → a backend `Slot` thread |

A `drain` binding runs its handler on a dedicated thread (`egui_mobius`
`Slot`), off the UI thread; it answers by writing shared `Dynamic<T>` state,
which the UI observes reactively. There is no reply edge — events go out,
results come back as state.

## 5. See the netlist, add a plugin

Everything the language decides is a flat, diffable netlist:

```shell
cargo run -p mobius_lang --example emit_ir -- crates/mobius_lang/examples/signal_bench.mobius
```

To let a `.mobius` file instantiate your own citizen crate, write a plugin —
three small pieces, walked through in
[`mobius_lang_host/PLUGINS.md`](../mobius_lang_host/PLUGINS.md).

Full language reference: `develop/mobius_lang.typ`.
