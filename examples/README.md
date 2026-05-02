# Examples

Working examples covering the three approaches in the `egui_mobius`
ecosystem: the citizen pattern (`egui_citizen`), the reactive
primitives (`egui_mobius_reactive`), and the older signal-slot
paradigm in `egui_mobius` core.

If you are new to the ecosystem, start with `getting_started` and
`reactive`. The first shows the citizen pattern end-to-end in a
single file; the second shows what `Dynamic<T>` actually does in
about twenty lines.

## Citizen pattern

- `getting_started` — three panels (Config, Display, Logger) wired
  through a dispatcher. Clicking a tab activates the citizen and the
  logger shows the lifecycle messages flowing through.
- `citizen_dock` — citizen + `egui_dock` with three algorithm tabs
  (Alpha, Beta, Gamma); the plot panel reacts to whichever tab is
  active, no per-frame fighting.
- **`filter_plotter` 🌐** — settings, plot, and logger panels driving
  a biquad lowpass filter backend. The book's tutorial walks through
  this example file by file. Runs both native and in the browser
  (WASM) — see `examples/filter_plotter/README.md`.
- `citizen_fetch` — backend thread doing HTTP fetches off the UI
  thread; image and response panels read the result reactively.

## Reactive primitives

- `reactive` — minimal `Dynamic<T>` counter. Read this first if you
  haven't used the reactive crate before.
- `reactive_slider` — `ReactiveWidgetRef` and weak references for
  composing widgets without cloning `Arc<T>` everywhere.
- `dashboard_async` — `Dynamic<T>` shared between the UI and an
  async background task running on `MobiusRuntime`.
- `clock_reactive` — full clock app: reactive state, multiple views,
  background processing through `MobiusRuntime`.

## Signal-slot / dispatcher

- `ui_refresh_events` — the smallest signal-slot example. Custom
  timed and programmatic UI refresh events.
- `dashboard` — `Dispatcher` pattern: register slots, send signals,
  separate UI from backend handlers.
- `clock_async` — signal-slot with `AsyncDispatcher` integrated
  against Tokio for concurrent processing.

## Components and streaming

- `logger_component` — uses `EventLogger` from
  `egui_mobius_components`. Multi-level severity, rich text,
  timestamp filtering.
- `realtime_plot` — streaming data visualization, mixing reactive
  state and signal-slot for the data path.

## Running

Each example is a workspace crate:

```bash
cargo run -p getting_started
cargo run -p reactive
cargo run -p filter_plotter
```

Most examples respect `RUST_LOG` for verbose output:

```bash
RUST_LOG=debug cargo run -p clock_reactive
```

### WASM (Browser) Support

🌐 **`filter_plotter`** can run in the browser:

```bash
cargo install trunk
rustup target add wasm32-unknown-unknown
cd examples/filter_plotter
trunk serve --open
```

See `examples/filter_plotter/README.md` for details. Other examples
require threading or async and would need core library changes to
support WASM.

## More

The book in `egui_mobius/book/` covers the citizen pattern and the
reactive primitives in depth. The
[`egui_mobius_template`](https://github.com/saturn77/egui_mobius_template)
repository is a starting scaffold for new applications.
