# Tutorial: Writing a citizen app

This chapter walks through `examples/filter_plotter/` end-to-end —
the project layout, every module, and how the citizen pattern wires
the pieces together. By the end you'll have written one citizen
app, and most of the scaffolding carries over directly to the next
one.

The app itself is small but realistic: a 50 Hz sine wave with
200 kHz noise added, run through a Butterworth biquad lowpass
filter, plotted with linked-axis subplots (matplotlib-style).
Three panels — a stacked input/output plot, a settings panel with
sliders, and a scrolling log panel — wired together by the
dispatcher.

> **Run it now**
>
> ```bash
> cargo run -p filter_plotter
> ```
>
> Click **Generate** in the Settings panel. The noisy input on
> top gets cleaned up in the filtered output below. Drop the
> cutoff slider and click Generate again to see the noise creep
> back in.

## The reusable scaffolding

Before the code, the punchline of the citizen pattern: most of
what you're about to build *also fits the next app you build*.
When you start the next citizen app, these files change very
little:

- `dispatcher.rs` — register citizens, drain messages, route AppMessage
- `tabs.rs` — the TabKind enum, Tab struct, TabViewer impl
- `messages.rs` — the AppMessage enum (specifically the Citizen variant)
- `main.rs` — App struct + drain loop pattern
- `state.rs` — SharedState shape with reactive parameters
- `theme.rs` — visuals + font scaling

What does change app-to-app:

- The contents of the panels (`panels/`)
- The backend (`backend/`)
- The non-Citizen variants of `AppMessage`

The dispatcher's plumbing is the part that scales sideways — write
it once and you're 80% of the way through every future citizen app.

## Project layout

```text
examples/filter_plotter/
├── Cargo.toml
└── src/
    ├── main.rs              # eframe::App, dock layout, drain loop
    ├── theme.rs             # apply_visuals, apply_font_scale
    ├── tabs.rs              # TabKind, Tab, TabViewer
    ├── messages.rs          # AppMessage enum
    ├── dispatcher.rs        # register / drain / handle
    ├── state.rs             # SharedState, ParamsState
    ├── backend/
    │   ├── mod.rs           # BackendKind trait, FilterParams, Traces
    │   └── iir.rs           # InProcessIir biquad lowpass
    └── panels/
        ├── mod.rs
        ├── plot.rs          # linked stacked plots
        ├── settings.rs      # sliders + Generate button
        └── logger.rs      # log scrollback
```

Each file has one job. The settings panel doesn't know how the
filter works; the backend doesn't know what egui is. The dispatcher
routes messages between them.

## The shape

The data flow on a "Generate" click:

```text
[Settings panel] ── click ──> AppMessage::Generate
                                │
                                v  (settings.outbox)
[main.rs drain loop] ── handle Generate ──> backend.run(params)
                                              │
                                              v
                                      [SharedState::traces]
                                              │
                                              v
                                  [Plot panel reads + renders]
```

The settings panel does **not** call the backend directly. It
pushes a message into its outbox; the drain loop in
`main.rs::update()` picks it up, calls the backend, stores the
result in `SharedState`, and the plot panel renders it on the next
frame.

## Shared state — `state.rs`

```rust,ignore
use eframe::egui;
use egui_mobius_reactive::Dynamic;

use crate::backend::{FilterParams, Traces};

pub struct ParamsState {
    pub signal_freq_hz: Dynamic<f32>,
    pub noise_freq_hz:  Dynamic<f32>,
    pub cutoff_hz:      Dynamic<f32>,
    pub sample_rate_hz: Dynamic<f32>,
    pub duration_ms:    Dynamic<f32>,
}

pub struct SharedState {
    pub params: ParamsState,
    pub traces: Dynamic<Traces>,
    pub log: Dynamic<Vec<String>>,
    pub plot_link: egui::Id,
}
```

Three reactive fields (`Dynamic<T>`) for things multiple places
read or write: `params` (settings panel writes, backend reads on
Generate), `traces` (drain loop writes, plot panel reads), `log`
(drain loop writes, logger panel reads). One non-reactive field
`plot_link` because both plot widgets only need the same `Id`; it
never changes after construction.

This is the **app-shared services** bucket from
[Where does state live?](../concepts/citizen.md#where-does-state-live)
in concrete form.

## The backend — `backend/`

The `BackendKind` trait abstracts what generates samples:

```rust,ignore
pub trait BackendKind {
    fn run(&mut self, params: &FilterParams) -> Traces;
    fn name(&self) -> &'static str;
}
```

The tutorial ships `InProcessIir` (in `backend/iir.rs`), which
generates a sine wave, adds a 200 kHz tone for noise, applies a
biquad lowpass, and returns both traces. A `SerialPort` impl
would have the same shape: `run` reads samples off a port; `name`
reports `"serial port"` instead. **The rest of the app — settings
panel, plot panel, dispatcher — does not change.** Swap the
backend type, the wiring stays.

The biquad itself is direct-form-II-transposed Butterworth
(Q = 1/√2):

```rust,ignore
fn lowpass(cutoff_hz: f32, sample_rate_hz: f32) -> Self {
    let q = std::f32::consts::FRAC_1_SQRT_2;
    let omega = 2.0 * std::f32::consts::PI * cutoff_hz / sample_rate_hz;
    let cos_w = omega.cos();
    let alpha = omega.sin() / (2.0 * q);

    let b0 = (1.0 - cos_w) / 2.0;
    /* ... a0/a1/a2 via bilinear transform ... */

    Self { /* normalized coefficients */ }
}

fn process(&mut self, x: f32) -> f32 {
    let y = self.b0 * x + self.z1;
    self.z1 = self.b1 * x + self.z2 - self.a1 * y;
    self.z2 = self.b2 * x - self.a2 * y;
    y
}
```

Standard textbook biquad — see the file for the full coefficients.
The math is incidental to the tutorial; the point is that this
struct lives in `backend/iir.rs` and only the `BackendKind` trait
crosses the module boundary.

## The settings panel — `panels/settings.rs`

```rust,ignore
pub struct SettingsPanel {
    pub citizen_id: CitizenId,
    pub citizen_state: CitizenState,
    pub outbox: Vec<AppMessage>,
}
```

Three fields. `citizen_id` and `citizen_state` are the boilerplate
that lets the dispatcher route activation to this panel. The
interesting one is `outbox`: a `Vec<AppMessage>` the panel pushes
to when something interesting happens, drained each frame by
`main.rs`.

The Generate button is one line:

```rust,ignore
if ui.add_sized([ui.available_width(), 28.0],
                egui::Button::new("Generate")).clicked() {
    self.outbox.push(AppMessage::Generate);
}
```

The panel does not call `backend.run()` directly. It does not call
`dispatcher.send()` either. It just enqueues an `AppMessage` for
the drain loop to handle. This keeps `show()` free of dependencies
on the backend or the dispatcher's internals — the panel is
testable in isolation, the message is the contract.

The sliders update reactive parameters via the standard get / set
loop:

```rust,ignore
let mut cutoff = state.params.cutoff_hz.get();
if ui.add(egui::Slider::new(&mut cutoff, 100.0..=50_000.0)
            .text("Lowpass cutoff (Hz)")
            .logarithmic(true))
        .changed()
{
    state.params.cutoff_hz.set(cutoff);
}
```

Read the current value from the `Dynamic<f32>`, hand egui a `&mut`
local, on change push the local back. Verbose, but transparent —
nothing is happening behind a wrapper.

## The plot panel — `panels/plot.rs`

```rust,ignore
const PLOT_STRIDE: usize = 50;

impl PlotPanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let traces = state.traces.get();
        if traces.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Click Generate to compute traces.");
            });
            return;
        }

        let half = (ui.available_height() - 8.0).max(120.0) / 2.0;

        ui.allocate_ui([ui.available_width(), half].into(), |ui| {
            Plot::new("input_plot")
                .link_axis(state.plot_link, [true, false])
                .height(half)
                .show(ui, |plot_ui| {
                    let pts: PlotPoints = traces.time.iter()
                        .zip(traces.input.iter())
                        .step_by(PLOT_STRIDE)
                        .map(|(&t, &y)| [t, y])
                        .collect();
                    plot_ui.line(Line::new("input", pts));
                });
        });

        // ...same shape for the filtered output...
    }
}
```

Two key bits:

1. **Linked axes via the same `egui::Id`.** Both `Plot::new(...)`
   calls pass `state.plot_link` (the same `Id`) to `link_axis`,
   so panning or zooming the input plot drives the filtered plot
   too. That's the matplotlib-style behavior.
2. **Decimation.** The backend computes 100,000 samples (1 MHz ×
   100 ms). Rendering all of them is wasteful — every 50th sample
   looks identical to the eye. `step_by(PLOT_STRIDE)` cheaply
   produces 2,000 plot points per trace.

The panel reads `state.traces` once at the top, holds the result
locally for the rest of the frame.

## The logger panel — `panels/logger.rs`

The simplest panel. Reads `state.log`, prints each line in a
scrolling area:

```rust,ignore
let log = state.log.get();
egui::ScrollArea::vertical()
    .auto_shrink([false, false])
    .stick_to_bottom(true)
    .show(ui, |ui| {
        if log.is_empty() {
            ui.weak("(no events yet)");
        } else {
            for line in log.iter() {
                ui.monospace(line);
            }
        }
    });
```

The log is populated entirely from the drain loop in
`dispatcher.rs::handle()`. The logger panel doesn't write to it
— it just renders.

## The dispatcher module — `dispatcher.rs`

This is where the pattern earns its name. Three jobs:

```rust,ignore
pub fn register_citizens(dispatcher: &mut Dispatcher) -> RegisteredCitizens {
    let plot     = dispatcher.register(CitizenId::new(PLOT_ID));
    let settings = dispatcher.register(CitizenId::new(SETTINGS_ID));
    let logger   = dispatcher.register(CitizenId::new(LOGGER_ID));
    dispatcher.activate(&CitizenId::new(PLOT_ID));
    RegisteredCitizens { plot, settings, logger }
}

pub fn drain_citizen(dispatcher: &mut Dispatcher, log: &Dynamic<Vec<String>>) {
    for msg in dispatcher.drain_messages() {
        append_log(log, format_citizen(&msg));
    }
}

pub fn handle<B: BackendKind>(
    msg: AppMessage,
    state: &SharedState,
    backend: &mut B,
    log: &Dynamic<Vec<String>>,
) {
    match msg {
        AppMessage::Citizen(_) => {} // already drained directly
        AppMessage::Generate => {
            let params = state.params.snapshot();
            let traces = backend.run(&params);
            let n = traces.input.len();
            state.traces.set(traces);
            append_log(log, format!("[INFO] backend ({}) produced {} samples",
                                    backend.name(), n));
        }
        AppMessage::GenerateCompleted { samples } => {
            append_log(log, format!("[INFO] generate completed: {} samples", samples));
        }
    }
}
```

`register_citizens` runs once at startup. `drain_citizen` and
`handle` run once per frame. `handle` is generic over backend
shape (`B: BackendKind`), which is what makes the dispatcher truly
app-agnostic — the same module would work with a `SerialPort`
backend, a `CsvImporter`, or anything else implementing
`BackendKind`.

## Wiring it together — `main.rs`

```rust,ignore
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.dock_state).show(ctx, &mut TabViewer {
            state: &self.state,
            dispatcher: &mut self.dispatcher,
            plot: &mut self.plot,
            settings: &mut self.settings,
            logger: &mut self.logger,
        });

        dispatcher::drain_citizen(&mut self.dispatcher, &self.state.log);

        let outbox = std::mem::take(&mut self.settings.outbox);
        for msg in outbox {
            dispatcher::handle(msg, &self.state, &mut self.backend, &self.state.log);
        }
    }
}
```

Five lines of orchestration:

1. Hand the dock area to `egui_dock` with our `TabViewer`.
2. Drain citizen activation messages into the log so the logger
   panel can show them.
3. Take the settings panel's outbox.
4. Process each `AppMessage` through `handle`.

That's it. **Adding a new panel** means: register it, add a
`TabKind` variant, wire it into `TabViewer::ui()`. **Adding a new
domain message** means: a new variant in `AppMessage` and a match
arm in `handle`. Neither is a refactor.

## Where to take it next

Concrete extensions, ordered by ambition:

- **Replace `InProcessIir` with `SerialPort`.** Implement
  `BackendKind::run` to read N samples off a port instead of
  generating them locally. UI doesn't change.
- **Stream the data instead of snapshotting.** Spawn a worker
  thread in a `Backend::start()` method, push samples through a
  channel, drain the channel into a ring buffer in `SharedState`
  each frame. Change `AppMessage::Generate` to `Start` / `Stop`.
  Plot panel reads the ring buffer.
- **Add a second filter stage.** Two `Biquad` sections cascaded
  for a 4th-order filter; expose a "filter order" combo box in
  Settings.
- **Persist the filter coefficients.** Write `ParamsState` to a
  RON file when the app exits, restore on startup. Routes through
  another `AppMessage::Save` / `Load`.

Each of these is one or two new modules and zero changes to the
`dispatcher.rs`, `tabs.rs`, or `main.rs` scaffolding. That's the
citizen pattern's transplant value, made concrete.

## Source

Full source lives in
[`examples/filter_plotter/`](https://github.com/saturn77/egui_mobius/tree/master/examples/filter_plotter)
in the `egui_mobius` repo. Run `cargo run -p filter_plotter` from
the workspace root.
