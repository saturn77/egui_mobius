<div align="center">
<img width=260 height=200 src="https://raw.githubusercontent.com/saturn77/egui_mobius/master/assets/mobius_strip.png"></img>

# egui_mobius
*Because GUI software design is a two sided problem operating on a single surface.*

[![egui](https://img.shields.io/badge/egui-0.33-blue)](https://github.com/emilk/egui)
[![egui_dock](https://img.shields.io/badge/egui__dock-0.18-purple)](https://github.com/Adanos020/egui_dock)
[![Crates.io](https://img.shields.io/crates/v/egui_mobius.svg)](https://crates.io/crates/egui_mobius)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml)
[![Book](https://img.shields.io/badge/📖_book-egui--mobius_%26%26_egui--citizen-orange)](https://saturn77.github.io/egui_mobius/)
![Rust 2024](https://img.shields.io/badge/rust-2024-blue.svg)

</div>

`egui_mobius` is a workspace of coordinated crates for building
professional `egui` applications — multi-panel docked layouts,
reactive shared state, and async backends — without each app
inventing its own organizational scheme.

The headline is the **`egui_citizen` pattern**: persistent panel
identity, reactive lifecycle state, and a central dispatcher for
panel priority and message routing. Underneath, `egui_mobius_reactive`
provides `Dynamic<T>` and `Derived<T>` — the cross-panel coupling
primitives. `egui_mobius` itself provides the signal-slot bus and an
async dispatcher for cross-thread work.

The [book](https://saturn77.github.io/egui_mobius/) covers the design
and walks through the examples end-to-end.

## Three levels of mobius-citizen apps

Most apps fit one of three architectural levels. Pick the one that
matches your app's complexity — the ecosystem supports each:

| Level | What you reach for | When | Examples |
|------:|--------------------|------|----------|
| **1** | Shared `Dynamic<T>` + `Dispatcher` for panel activation | Pure UI; no backend | `getting_started`, `citizen_dock` |
| **2** | Above + `AppMessage` routed through `Dispatcher::handle` | Synchronous backend (filter, parser, anything in-process) | `filter_plotter`, `citizen_fetch` |
| **3** | Above + `egui_mobius` signal/slot + `AsyncDispatcher` | Async / multi-threaded backend | `citizen_signal_async` |

`filter_plotter` is the tutorial example — the book walks through
it file by file.

## Quick start

```bash
# Level 1 — pure UI / panel coupling
cargo run -p getting_started       # smallest possible citizen app
cargo run -p citizen_dock          # citizen + egui_dock, three panels

# Level 2 — shared state + backend routing
cargo run -p filter_plotter        # tutorial: biquad filter + plotter
cargo run -p citizen_fetch         # backend thread doing HTTP fetches

# Level 3 — signals/slots + async
cargo run -p citizen_signal_async  # citizen + signal/slot + Tokio backend
```

Other examples (reactive primitives in isolation, signal/slot
without citizens, etc.) are catalogued in
[`examples/README.md`](examples/README.md).

## The workspace

| Crate | Role |
|-------|------|
| `egui_citizen` | The citizen *pattern* — panel identity, reactive lifecycle state, dispatcher for activation arbitration and message routing. Where most app code sits. |
| `egui_mobius_reactive` | Thread-safe reactive primitives: `Dynamic<T>` for shared cells, `Derived<T>` for auto-recomputed values. The cross-panel coupling layer. |
| `egui_mobius` | Signal/slot bus + `AsyncDispatcher` for cross-thread async backends. Needed at level 3. |
| `egui_mobius_widgets` | Stateful widget toolkit for retained-mode-style composition. |
| `egui_mobius_components` | Higher-level UI components (event logger, etc.). |

## See also

- **[egui_mobius_template](https://github.com/saturn77/egui_mobius_template)** — project skeleton for new applications.
- **[CopperForge](https://github.com/Atlantix-EDA/CopperForge)** — real-world reference implementation: a PCB gerber inspection tool built end-to-end on the citizen pattern.

## Contributing

- Contributions welcome — fork, branch, PR.
- Licensed under MIT.
- Questions or discussion: open an issue or use [GitHub Discussions](https://github.com/saturn77/egui_mobius/discussions).
