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
registry.

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

> **Run it in the browser (WASM)**
>
> `filter_plotter` is also the workspace's reference implementation
> for browser deployment. One-time setup:
>
> ```bash
> cargo install trunk
> rustup target add wasm32-unknown-unknown
> ```
>
> Then, from the example directory:
>
> ```bash
> cd examples/filter_plotter
> trunk serve --open       # development; opens http://127.0.0.1:8080
> trunk build --release    # production; output in ./dist/
> ```
>
> The release `dist/` directory is a self-contained static site —
> drop it on any web host. Everything in this tutorial works in
> the browser identically: the citizen pattern, the registry,
> the reactive cells, the IIR backend. The only platform-specific
> code is the `#[cfg(target_arch = "wasm32")]` entrypoint in
> `main.rs` that hands eframe a canvas instead of a native
> window. See `examples/filter_plotter/README.md` for the full
> wasm story.

## The reusable scaffolding

Before the code, the punchline of the citizen pattern: most of
what you're about to build *also fits the next app you build*.
When you start the next citizen app, these files change very
little:

- `citizens/mod.rs` — registry wiring: the lifecycle drain, log helpers
- `tabs.rs` — the TabKind enum, Tab struct, TabViewer impl, id consts
- `messages.rs` — the AppMessage enum
- `main.rs` — App struct, registration, drain loop
- `state.rs` — SharedState shape with reactive parameters
- `theme.rs` — visuals + font scaling

What does change app-to-app:

- The citizens themselves (`citizens/<name>.rs`)
- Their actions (`<name>_actions.rs`)
- The backend (`backend/`)
- The variants of `AppMessage`

The registry wiring is the part that scales sideways — write it
once and you're 80% of the way through every future citizen app.

## Project layout

One file per citizen under `citizens/`, and one `<citizen>_actions.rs`
per citizen that emits app-level messages. Citizens that only render
(plot, logger, editor) need no actions file.

```text
examples/filter_plotter/
├── Cargo.toml
└── src/
    ├── main.rs              # eframe::App, registration, dock, drain loop
    ├── theme.rs             # apply_visuals, apply_font_scale
    ├── tabs.rs              # TabKind, Tab, TabViewer, id consts
    ├── messages.rs          # AppMessage enum
    ├── settings_actions.rs  # settings' outbox → backend (the dispatch path)
    ├── state.rs             # SharedState, ParamsState
    ├── backend/
    │   ├── mod.rs           # BackendKind trait, FilterParams, Traces
    │   └── iir.rs           # InProcessIir biquad lowpass
    └── citizens/
        ├── mod.rs           # registry wiring: drain_citizen, append_log
        ├── plot.rs          # linked stacked plots
        ├── settings.rs      # sliders + Generate button
        ├── logger.rs        # log scrollback
        └── editor.rs        # egui_quill editor tab
```

Each file has one job. The settings citizen doesn't know how the
filter works; the backend doesn't know what egui is. The actions
module between them is the only place that touches both — and it is
the only file in the app doing *dispatch*: sending data to the
backend. The registry never does; its wiring in `citizens/mod.rs` is
lifecycle only.

## The shape

The data flow on a "Generate" click:

```text
[Settings citizen] ── click ──> AppMessage::Generate
                                  │
                                  v  (settings.outbox)
[main.rs drain loop] ── settings_actions::handle ──> backend.run(params)
                                                       │
                                                       v
                                               [SharedState::traces]
                                                       │
                                                       v
                                           [Plot citizen reads + renders]
```

The settings citizen does **not** call the backend directly. It
pushes a message into its outbox; the drain loop in
`main.rs::update()` hands it to `settings_actions::handle`, which
calls the backend and stores the result in `SharedState`; the plot
citizen renders it on the next frame.

## Shared state — `state.rs`

```rust,ignore
use eframe::egui;
use egui_lens::{LogColors, ReactiveEventLoggerState};
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
    pub traces: Dynamic<Traces<f32>>,
    pub log: Dynamic<ReactiveEventLoggerState>,
    pub log_colors: Dynamic<LogColors>,
    pub plot_link: egui::Id,
}
```

Four reactive fields (`Dynamic<T>`) for things multiple places
read or write: `params` (settings panel writes, backend reads on
Generate), `traces` (drain loop writes, plot panel reads), `log`
(drain loop writes, logger panel reads — backed by
`egui_lens::ReactiveEventLoggerState`, see [the lens
chapter](../concepts/lens.md)), and `log_colors` (configures
per-type colors for custom log channels like `"citizen"` and
`"backend"`). One non-reactive field `plot_link` because both plot
widgets only need the same `Id`; it never changes after
construction.

The `f32` in `Dynamic<Traces<f32>>` is where this app commits to
the in-process IIR backend's sample type — see
[The backend](#the-backend--backend) below for why `Traces<T>` is
generic and how a fixed-point backend would change just this one
type.

This is the **app-shared services** bucket from
[Where does state live?](../concepts/citizen.md#where-does-state-live)
in concrete form.

## The backend — `backend/`

Two plain data types and a trait. The data types come first
because the trait signature uses them.

```rust,ignore
/// Parameters captured at "Generate" time — a snapshot of the reactive
/// fields on `SharedState::params` so the backend has a stable, owned
/// view of what to compute.
#[derive(Debug, Clone, Copy)]
pub struct FilterParams {
    pub signal_freq_hz:  f32,
    pub noise_freq_hz:   f32,
    pub noise_amplitude: f32,
    pub cutoff_hz:       f32,
    pub sample_rate_hz:  f32,
    pub duration_ms:     f32,
}

impl FilterParams {
    pub fn num_samples(&self) -> usize {
        (self.sample_rate_hz * self.duration_ms / 1000.0).round() as usize
    }
}

/// One pair of traces resulting from a Generate run.
///
/// `T` is the sample type. The in-process IIR backend uses `f32`; a
/// serial-port backend feeding raw ADC counts could use `i16` or `i32`
/// without a lossy upcast at the boundary. Time stays `f64` regardless
/// — timestamps are the same kind of value across all backends.
#[derive(Debug, Clone)]
pub struct Traces<T> {
    pub time:     Vec<f64>,  // seconds
    pub input:    Vec<T>,    // raw noisy signal
    pub filtered: Vec<T>,    // lowpass output
}

impl<T> Default for Traces<T> {
    fn default() -> Self {
        Self {
            time:     Vec::new(),
            input:    Vec::new(),
            filtered: Vec::new(),
        }
    }
}

impl<T> Traces<T> {
    pub fn is_empty(&self) -> bool { self.time.is_empty() }
}
```

Three things worth pointing out:

- **`FilterParams` is `Copy`** because it's a plain bag of `f32`s.
  The settings panel writes reactive `Dynamic<f32>` fields; on
  Generate we *snapshot* them into a `FilterParams` (see
  `state.rs::ParamsState::snapshot()`) so the backend gets a
  stable, owned value. No reactivity crosses the trait boundary.
- **`Traces<T>` is generic over the sample type, not the time type.**
  This is the difference between an emulator backend producing
  `f32` samples and a serial-port backend producing `i16` ADC
  counts — neither needs to lossily upcast at the boundary. The
  *time* axis stays `Vec<f64>` because seconds-since-start is the
  same kind of quantity everywhere; only the sample magnitudes
  vary in representation.
- **`Traces` uses parallel `Vec`s columnar**, three vectors of the
  same length, not a `Vec<(f64, T, T)>` of points. Columnar is
  what the plot library wants — `traces.time.iter()
  .zip(traces.input.iter())` to produce points only at render time
  — and it's what a streaming backend would also produce. Keeping
  the shape columnar from day one means swapping in a streaming
  backend later doesn't reshape the data.

`Default` is implemented manually instead of derived because the
derive would inject a spurious `T: Default` bound; `Vec::new()`
doesn't need it.

The `BackendKind` trait then abstracts what produces a `Traces`
from a `FilterParams`. The sample type is an associated type, so
each backend names exactly one:

```rust,ignore
pub trait BackendKind {
    type Sample;
    fn run(&mut self, params: &FilterParams) -> Traces<Self::Sample>;
    fn name(&self) -> &'static str;
}
```

The tutorial ships `InProcessIir` (in `backend/iir.rs`) with
`type Sample = f32;` — it generates a sine wave, adds a 200 kHz
tone for noise, applies a biquad lowpass, and returns both traces
as `Traces<f32>`. A `SerialPort` impl would set
`type Sample = i16;` (or whatever the ADC width is) and `run`
would read samples off a port. **The rest of the app — settings
citizen, plot citizen, actions — does not change shape.** Swap the
backend type and the wiring stays.

The one place that *commits* to a sample type is `SharedState`:
```rust,ignore
pub traces: Dynamic<Traces<f32>>,
```
The reactive cell has to hold a concrete `T`. Using a different
backend means changing this `f32` to match
`Backend::Sample`, but `settings_actions::handle` uses
`B: BackendKind<Sample = f32>` to enforce the match at compile
time, so the wiring stays honest.

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

## The settings citizen — `citizens/settings.rs`

```rust,ignore
use crate::tabs::SETTINGS_ID;
use egui_citizen::citizen_panel;

citizen_panel!(SettingsPanel, SETTINGS_ID, outbox: Vec<AppMessage> = Vec::new());
```

One declaration. The macro generates the struct — `citizen_id` and
`citizen_state` boilerplate plus the declared field — the
`new(citizen_state)` constructor, and the `Citizen` impl. The
interesting field is `outbox`: a `Vec<AppMessage>` the citizen
pushes to when something interesting happens, drained each frame by
`main.rs` into `settings_actions::handle`.

The Generate button is one line:

```rust,ignore
if ui.add_sized([ui.available_width(), 28.0],
                egui::Button::new("Generate")).clicked() {
    self.outbox.push(AppMessage::Generate);
}
```

The panel does not call `backend.run()` directly. It does not call
`registry.send()` either. It just enqueues an `AppMessage` for
the drain loop to handle. This keeps `show()` free of dependencies
on the backend or the registry's internals — the panel is
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

## The plot citizen — `citizens/plot.rs`

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

## The logger citizen — `citizens/logger.rs`

The logger panel is itself a **citizen with named atoms** —
`egui_lens::ReactiveEventLogger` provides the panel identity (the
"logger" citizen) and the in-panel widgets (System Info, Filters,
Logger Colors, Save Logs, Clear Logs, the column-toggle
checkboxes, the scrollable log area) are its atoms. The panel
struct in this app is a thin wrapper that hands lens its shared
state and renders it:

```rust,ignore
use egui_lens::ReactiveEventLogger;
use crate::state::SharedState;

pub struct LoggerPanel {}

impl LoggerPanel {
    pub fn new() -> Self { Self {} }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let logger = ReactiveEventLogger::with_colors(
            &state.log,
            &state.log_colors,
        );
        logger.show(ui);
    }
}
```

The logger reads `state.log` (a `Dynamic<ReactiveEventLoggerState>`)
and `state.log_colors` (a `Dynamic<LogColors>`); writes flow in
through the drain loop. The logger citizen doesn't push entries
itself — it just renders what `citizens::drain_citizen` and
`settings_actions::handle` put there.

> *Forward-looking note:* lens will eventually implement
> `Citizen` directly and the `LoggerPanel` wrapper here
> disappears entirely — the dock layout will use the citizen
> `ReactiveEventLogger`, or its sibling `LoggerCitizen`, without
> any custom panel type. For now this thin wrapper is the
> boundary.

### What the lens-backed logger gives you

Compared to a hand-rolled `Vec<String>` log:

- **Per-type colors** via `Dynamic<LogColors>` — the drain routes
  citizen lifecycle events through `log_custom("citizen",
  ...)` and the actions route backend events through
  `log_custom("backend", ...)`,
  each rendered in its own color (configured at app construction
  in `state.rs`).
- **Filters** — info / warning / error / debug / custom / system
  toggles plus a text-search box, all stored on the shared state.
- **Save Logs** — exports current contents via
  `rfd::AsyncFileDialog`, which works on **both native and WASM**
  (browser builds get a download; native opens an OS file
  dialog).
- **System Info button** — sets a memory flag the consuming app
  drains. The "atom" pattern at work: lens publishes the click;
  the app provides the actual implementation (see
  `platform/details.rs` for native, `platform/details_wasm.rs`
  for the browser variant).
- **Cross-thread writes** — `Dynamic::clone()` is cheap; hand a
  clone to any thread and call `ReactiveEventLogger::new(&clone)
  .log_info(...)` from anywhere.

## Registry wiring — `citizens/mod.rs`

Everything registry-side lives with the citizens: the module decls
and the lifecycle drain. Note what is *not* here — nothing in this
file talks to the backend.

```rust,ignore
pub mod editor;
pub mod logger;
pub mod plot;
pub mod settings;

/// Drain citizen lifecycle messages from the registry and route them
/// into the shared lens-backed log. Call once per frame after
/// `DockArea::show`.
pub fn drain_citizen(registry: &mut Registry, state: &SharedState) {
    let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
    for msg in registry.drain_messages() {
        // "citizen" is a named custom log type — its color is
        // configured in state.rs and renders distinctly from
        // info/warning/error/debug.
        logger.log_custom("citizen", &format_citizen(&msg));
    }
}
```

Registration itself happens in `main.rs` at construction, where the
panels are built — each citizen holds the `CitizenState` handle the
registry gives back:

```rust,ignore
let mut registry = Registry::new();
let plot_state     = registry.add().with_name(tabs::PLOT_ID);
let settings_state = registry.add().with_name(tabs::SETTINGS_ID);
let logger_state   = registry.add().with_name(tabs::LOGGER_ID);
let editor_state   = registry.add().with_name(tabs::EDITOR_ID);
registry.activate(tabs::PLOT_ID);
```

## Actions — `settings_actions.rs`

One `<citizen>_actions.rs` per citizen that emits `AppMessage`s.
This is the dispatch path — the only file in the app where the word
applies, because it is the one place data is sent toward the
backend. The settings citizen is the only emitter in this app, so
there is exactly one actions file:

```rust,ignore
pub fn handle<B>(msg: AppMessage, state: &SharedState, backend: &mut B)
where
    B: BackendKind<Sample = f32>,
{
    let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
    match msg {
        AppMessage::Generate => {
            let params = state.params.snapshot();
            let traces = backend.run(&params);
            let n = traces.input.len();
            state.traces.set(traces);
            // "backend" is also a named custom log type — its
            // color is configured alongside "citizen" so the two
            // event streams are visually distinct.
            logger.log_custom("backend",
                &format!("{} produced {} samples", backend.name(), n));
        }
    }
}
```

Registration runs once at startup. `drain_citizen` and `handle` run
once per frame. `handle` is generic over backend shape
(`B: BackendKind<Sample = f32>`), which is what keeps the actions
app-agnostic at the *behavior* layer — the same module would work
with a `SerialPort` backend, a `CsvImporter`, or anything else
implementing `BackendKind` whose `Sample` matches what
`SharedState::traces` holds. Pinning the sample type at the `where`
clause means the compiler catches any backend swap that forgot to
update `SharedState`.

When a second citizen grows an outbox — say the editor learns to
apply scripts — it gets its own `editor_actions.rs`. Actions scale
per-citizen, not by growing one central module.

## The tabs module — `tabs.rs`

`egui_dock` needs two things from your app: a tab type, and a
`TabViewer` impl that knows how to render each tab. `tabs.rs` is
where both live, plus the citizen IDs that name each panel.

```rust,ignore
pub const PLOT_ID:     &str = "plot";
pub const SETTINGS_ID: &str = "settings";
pub const LOGGER_ID:   &str = "logger";

#[derive(Clone, Copy)]
pub enum TabKind {
    Plot,
    Settings,
    Logger,
}

pub struct Tab {
    pub kind: TabKind,
}

impl Tab {
    pub fn new(kind: TabKind) -> Self { Self { kind } }

    pub fn title(&self) -> &'static str {
        match self.kind {
            TabKind::Plot     => "Plot",
            TabKind::Settings => "Settings",
            TabKind::Logger   => "Logger",
        }
    }

    pub fn citizen_id(&self) -> CitizenId {
        CitizenId::new(match self.kind {
            TabKind::Plot     => PLOT_ID,
            TabKind::Settings => SETTINGS_ID,
            TabKind::Logger   => LOGGER_ID,
        })
    }
}
```

`TabKind` is the closed set of panels the app knows about. `Tab`
wraps a `TabKind` because `egui_dock::DockState<T>` stores `T`
directly — wrapping it in a struct gives us a stable place to hang
helpers like `title()` and `citizen_id()`. Adding a fourth panel is
one new variant plus a match arm in each helper; no other file
moves.

The `citizen_id()` method is what links `egui_dock`'s tab-click
event back into the citizen layer — clicking the Settings tab needs
to activate `CitizenId::new("settings")` so the registry knows
that panel is now in focus. Keeping the IDs as `pub const` strings
in this file means `main.rs`, the citizens, and `tabs.rs` agree by
import,
not by typo-prone string duplication.

### The `TabViewer` bridge

`egui_dock::TabViewer` is the trait the dock area calls into to
render each tab. Our impl is the *one place* in the app that holds
mutable references to every panel and the registry at once:

```rust,ignore
pub struct TabViewer<'a> {
    pub state: &'a SharedState,
    pub registry: &'a mut Registry,
    pub plot: &'a mut PlotPanel,
    pub settings: &'a mut SettingsPanel,
    pub logger: &'a mut LoggerPanel,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab.title())
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.kind {
            TabKind::Plot     => self.plot.show(ui, self.state),
            TabKind::Settings => self.settings.show(ui, self.state),
            TabKind::Logger   => self.logger.show(ui, self.state),
        }
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.clicked() {
            self.registry.activate(&tab.citizen_id());
        }
    }
}
```

Four methods, each doing one thing:

1. **`title`** — return whatever the tab strip should display.
   We delegate to `Tab::title()` so the strings live next to the
   enum.
2. **`id`** — a stable identity per tab, required since
   `egui_dock` 0.21. Derive it from the same string `title()`
   uses — but never from decorated display text.
3. **`ui`** — `egui_dock` calls this once per visible tab per
   frame. We match on `tab.kind` and dispatch to the corresponding
   panel's `show()`. Every panel sees `&SharedState` and nothing
   else — the registry stays up here at the app layer.
4. **`on_tab_button`** — fired when the user clicks a tab header.
   We forward the click into `registry.activate(...)`. This is
   the canonical citizen hook: `egui_dock` knows about the click;
   the registry knows about activation; this method is the
   bridge. Even if your app doesn't currently *do* anything on
   activation, register the click — the registry's queue stays
   accurate, and adding behavior later doesn't require revisiting
   this file.

The borrow story is worth a moment: `TabViewer` holds five `&mut`
references at once, which would be a problem in a long-lived
struct, but it's constructed inline in `update()` and dropped at
the end of `DockArea::show()`. Short-lived, single-frame — the
compiler is happy because no two methods on `TabViewer` borrow
overlapping fields.

### Why this file barely changes between apps

Look back at the *reusable scaffolding* list at the top of the
chapter. `tabs.rs` is on it because:

- The `TabKind` enum changes (new panels) but the *shape* doesn't.
- The `Tab` struct, `title()`, `citizen_id()` pattern is verbatim
  across apps.
- The `TabViewer` impl gains/loses fields as panels come and go,
  but the four methods (`title`, `id`, `ui`, `on_tab_button`) are
  always the same four methods doing the same four jobs.

Adding a panel is mechanical: new const, new `TabKind` variant,
new title, new `citizen_id` arm, new `&mut Panel` field on
`TabViewer`, new arm in `ui`. Six edits, no thinking — exactly
the kind of change the citizen pattern is designed to make boring.

## Wiring it together — `main.rs`

```rust,ignore
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.dock_state).show(ctx, &mut TabViewer {
            state: &self.state,
            registry: &mut self.registry,
            plot: &mut self.plot,
            settings: &mut self.settings,
            logger: &mut self.logger,
            editor: &mut self.editor,
        });

        // Drain pass — once per frame, after the dock has rendered and
        // any on_tab_button or in-panel events have queued.
        citizens::drain_citizen(&mut self.registry, &self.state);

        let outbox = std::mem::take(&mut self.settings.outbox);
        for msg in outbox {
            settings_actions::handle(msg, &self.state, &mut self.backend);
        }
    }
}
```

Five lines of orchestration:

1. Hand the dock area to `egui_dock` with our `TabViewer`.
2. Drain citizen activation messages into the log so the logger
   panel can show them.
3. Take the settings panel's outbox.
4. Process each `AppMessage` through `settings_actions::handle`.

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

Each of these is one or two new modules — a citizen file, perhaps
an actions file — and zero changes to the `citizens/mod.rs`,
`tabs.rs`, or `main.rs` scaffolding. That's the citizen pattern's
transplant value, made concrete.

## Source

Full source lives in
[`examples/filter_plotter/`](https://github.com/saturn77/egui_mobius/tree/master/examples/filter_plotter)
in the `egui_mobius` repo. Run `cargo run -p filter_plotter` from
the workspace root.
