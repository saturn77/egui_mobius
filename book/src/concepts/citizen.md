# The Citizen trait

The `Citizen` trait is how a panel struct *becomes* a citizen. It
gives the panel three things:

1. A persistent identity — a [`CitizenId`](#identities).
2. A handle to its [reactive lifecycle state](state.md) — a
   `CitizenState`.
3. Default lifecycle hooks (`on_activate`, `on_deactivate`,
   `on_click`) and reader convenience methods (`is_active`,
   `is_selected`).

This chapter covers the trait surface, the way state flows between
the dispatcher and the panel, where panel-author state lives across
the three structs that any non-trivial app uses, and the runtime
story for how the trait actually gets exercised.

## The trait

```rust,ignore
pub trait Citizen {
    fn id(&self) -> &CitizenId;
    fn citizen_state(&self) -> &CitizenState;
    fn citizen_state_mut(&mut self) -> &mut CitizenState;

    // Defaulted hooks (override if you need custom behavior):
    fn on_activate(&mut self)   { self.citizen_state_mut().active.set(true); }
    fn on_deactivate(&mut self) { self.citizen_state_mut().active.set(false); }
    fn on_click(&mut self)      { self.citizen_state_mut().clicked.set(true); }

    // Defaulted readers:
    fn is_active(&self)   -> bool { self.citizen_state().active.get() }
    fn is_selected(&self) -> bool { self.citizen_state().selected.get() }
}
```

Three required methods. Three defaulted hooks. Two defaulted readers.
That is the whole trait.

## A working panel

The trait's three required methods (`id`, `citizen_state`,
`citizen_state_mut`) are pure plumbing — they hand the trait
references back to the fields you store on the struct. They're
required because the dispatcher and the defaulted hooks need a
uniform way to reach into your panel; that's the entire purpose.

What the trait actually *buys* you is the rest of the contract:
`is_active()`, `is_selected()`, and the defaulted `on_activate` /
`on_deactivate` / `on_click` hooks. A panel that uses those is
the real minimum:

```rust,ignore
struct PlotPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    samples: Vec<f32>,
}

impl PlotPanel {
    fn new(state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new("plot"),
            citizen_state: state,
            samples: Vec::new(),
        }
    }

    fn show(&mut self, ui: &mut egui::Ui, dispatcher: &mut Dispatcher) {
        ui.heading("Plot");

        // Read direction: dispatcher → panel.
        // The dispatcher's activate() writes self.citizen_state.active
        // through the shared Arc; we observe it reactively via
        // is_active(). No method call from this side is needed.
        if self.is_active() {
            ui.label("(active — drawing live)");
            // ... actual plotting against self.samples ...
        } else {
            ui.label("(inactive — paused)");
        }

        // Write direction: panel → dispatcher.
        // The panel can hand control to a sibling citizen explicitly.
        if ui.button("Switch to settings").clicked() {
            dispatcher.activate(&CitizenId::new("settings"));
        }
    }
}

// The trait impl below is required boilerplate — three accessors that
// hand the trait its own data back. There's nothing interesting here.
impl Citizen for PlotPanel {
    fn id(&self) -> &CitizenId               { &self.citizen_id }
    fn citizen_state(&self) -> &CitizenState  { &self.citizen_state }
    fn citizen_state_mut(&mut self) -> &mut CitizenState {
        &mut self.citizen_state
    }
}
```

`is_active()` is what makes the panel *do* something — it's a
defaulted method on the trait that reads
`self.citizen_state().active.get()` for you. Without the trait,
you'd write that path manually every time you wanted to check
activation. The accessor boilerplate is the price; the readers
and hooks are what you actually buy.

The two state-flow directions are intentionally asymmetric.
**Dispatcher → panel happens reactively** — the dispatcher writes
through the `Arc` underneath `citizen_state.active`, and the panel
sees the new value the next time `is_active()` reads it. No
method call from the panel side is needed. **Panel → dispatcher
requires a method-call surface**, which is what `&mut Dispatcher`
in the `show()` signature provides — the panel can call
`dispatcher.activate(...)` to hand control to a sibling, or
`dispatcher.send(...)` to push a custom message. Some apps wrap
the dispatcher and shared services in a `PanelCtx` struct
(`fn show(&mut self, ui: &mut egui::Ui, ctx: &mut PanelCtx)`) to
keep the parameter list short; either shape works.

The `state` argument to `PlotPanel::new` should always come from
[`Dispatcher::register()`](dispatcher.md#registerid---citizenstate),
**never** from `CitizenState::new()` or `CitizenState::default()`.
The latter allocate fresh disconnected storage and silently sever
the reactive link with the dispatcher (see
[the trap in the state chapter](state.md#the-trap-that-bites-everyone)).

## Atoms — widget state alongside `CitizenState`

A citizen-panel almost always carries its own widget state: slider
values, combo-box selections, text-input buffers, checkbox flags.
The [introduction](../introduction.md#key-vocabulary) calls these
**atoms**. They live on the panel struct *alongside* `citizen_state`,
not inside it — `CitizenState` has a fixed library-defined shape and
is for lifecycle facts only. Where you place an atom depends on
whether anyone outside the panel reads or writes it.

### Atoms only the panel itself touches

These are plain (non-reactive) fields. The panel reads them in
`show()`, egui mutates them in place via `&mut`. Nothing fancy.

```rust,ignore
#[derive(Debug, Clone, PartialEq)]
enum PlotStyle { Line, Scatter, Bar }

struct PlotPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    samples: Vec<f32>,
    // Atoms — panel-local widget state:
    sample_rate_hz: f32,
    plot_style: PlotStyle,
    show_grid: bool,
}

impl PlotPanel {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Plot");

        ui.add(egui::Slider::new(&mut self.sample_rate_hz, 1.0..=1000.0)
            .text("Sample rate (Hz)"));

        egui::ComboBox::from_label("Style")
            .selected_text(format!("{:?}", self.plot_style))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.plot_style, PlotStyle::Line,    "Line");
                ui.selectable_value(&mut self.plot_style, PlotStyle::Scatter, "Scatter");
                ui.selectable_value(&mut self.plot_style, PlotStyle::Bar,     "Bar");
            });

        ui.checkbox(&mut self.show_grid, "Show grid");

        // ... draw the plot using these values ...
    }
}
```

Plain `f32`, plain `bool`, plain enum. The citizen layer never sees
them, doesn't care about them. This is the right shape for "only
this panel uses these values."

### Atoms another panel or thread reads

When something outside the panel needs the value — another panel
mirroring it, a backend thread parameterizing its work, a logger
recording every change — promote the field to a `Dynamic<T>` so it
can be cloned and shared:

```rust,ignore
use egui_mobius_reactive::Dynamic;

struct PlotPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    samples: Vec<f32>,
    // Reactive atom — other panels / threads can hold a clone:
    sample_rate_hz: Dynamic<f32>,
}

impl PlotPanel {
    fn show(&mut self, ui: &mut egui::Ui) {
        let mut local = self.sample_rate_hz.get();
        if ui
            .add(egui::Slider::new(&mut local, 1.0..=1000.0).text("Sample rate (Hz)"))
            .changed()
        {
            self.sample_rate_hz.set(local);
        }
        // ... rest of show() ...
    }
}
```

`Dynamic<f32>` is the same shape as the fields inside `CitizenState` —
an `Arc`-backed reactive cell. Cloning it gives another panel or
backend thread a handle to the same value (see
[Inside `Dynamic<T>`](inside-dynamic.md) for the mechanics, and
[Coupling paths](coupling.md) for how an atom can fan out to
UI-to-UI sharing, UI-to-backend messaging, or both at once).

### Don't reach for `Dynamic<T>` until a second reader exists

Reactivity has a real cost — every `Dynamic<T>` is an `Arc` plus a
lock plus a notifier list. If only the panel itself reads its slider
value, a plain `f32` is the right type. Promote to `Dynamic<f32>`
the day a second reader actually appears. Speculative reactivity
"in case someone needs this later" is the same kind of mistake as
speculative `Arc<Mutex<...>>` — it pays a cost for an option you
may never exercise.

## Where does state live?

The atoms section above showed the panel-local choice between a
plain field and a `Dynamic<T>`. Step back, and that's part of a
broader three-struct model that any non-trivial citizen-panel app
converges on.

### The three structs

#### 1. `CitizenState` — lifecycle facts only

The library type. Six fixed `Dynamic<T>` fields, reactive by design,
shared by `Arc`. **You cannot extend it** — it has a fixed contract.

What goes here: questions other panels or the dock ask about *this
panel's status* (active, clicked, selected, moved, location,
visible).

What does **not** go here: business data, widget values, anything
outside the six lifecycle facts.

#### 2. `PanelState` — your panel-local struct

Whatever the panel needs to do its own job that nobody else reads
or writes. By convention, give it its own struct named
`FooPanelState` (or just `PanelState` if scoped inside a panel
module):

```rust,ignore
struct LoggerPanelState {
    log_buffer: Vec<LogEntry>,
    filter_text: String,
    follow_tail: bool,
}

struct LoggerPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,    // bucket 1: library lifecycle
    panel_state: LoggerPanelState,  // bucket 2: panel-local data
}
```

Atoms (slider values, combo-box selections, checkbox flags) live
here when only the panel reads them — as plain fields, not
`Dynamic<T>`. Promote to `Dynamic<T>` only when a second reader
shows up.

Why a named struct instead of loose fields on the panel? Three
reasons:

1. **It names the bucket.** A reader scanning `LoggerPanel` sees
   three things — id, citizen state, panel state — instead of a
   flat list that mixes concerns.
2. **It mirrors `CitizenState`.** Both are "state for one panel,"
   one library-defined and one app-defined. The parallel makes the
   design rule visible.
3. **It survives refactors.** When the panel grows, panel-local
   fields stay clustered. When you eventually need to persist or
   snapshot panel state, it's already a single value.

For tiny panels with one or two fields, inlining on the panel
struct is fine — promote to a named struct the moment a third
field appears.

#### 3. App-shared state — data many panels touch

Anything two or more panels need to read or mutate. Project config,
a database handle, the layer store in a CAD app, a theme. Lives at
the app level and gets passed by reference into each panel's
`show()`:

```rust,ignore
struct App {
    services: Arc<SharedServices>,
    dispatcher: Dispatcher,
    logger: LoggerPanel,
    bom: BomPanel,
}

// And inside each panel's show():
self.logger.show(ui, &mut self.dispatcher, &self.services);
self.bom.show(ui, &mut self.dispatcher, &self.services);
```

Whether the shared bits are themselves reactive (`Dynamic<T>`
inside `SharedServices`) or guarded by `Arc<Mutex<...>>` is a
separate design choice. The point: shared data lives at the app
level, not stuffed inside `CitizenState` and not duplicated across
panel structs.

### The rule of thumb

Ask in this order:

| Question                          | Bucket               |
|-----------------------------------|----------------------|
| Is this a lifecycle fact?         | `CitizenState`       |
| Do two or more panels need it?    | App-shared           |
| Otherwise                         | `PanelState`         |

If a piece of data fits the lifecycle list — active, clicked,
selected, moved, location, visible — it belongs in `CitizenState`.
If not, ask whether anything outside the panel reads or mutates
it. If yes, app-shared. Otherwise, `PanelState`.

### Anti-patterns

**Trying to extend `CitizenState` with business fields.** It has a
fixed shape from the library. Wrap it inside your own panel struct
alongside your fields; don't try to extend it.

```rust,ignore
// WRONG — won't compile, and shouldn't
struct CitizenState {
    active: Dynamic<bool>,
    // ...
    bom_rows: Vec<BomRow>, // no
}

// RIGHT
struct BomPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    panel_state: BomPanelState,    // bom_rows lives in here
}
```

**Duplicating shared data across panels.** If three panels need to
read the project config, don't give each one its own copy and try
to synchronize. Put it in `SharedServices` and pass `&services`
into each panel's `show()`.

**Putting `PanelState` fields in `Dynamic<T>` "in case someone
needs them later."** Reactivity has real cost (every `Dynamic<T>`
is an `Arc` plus a lock plus a notifier list). Promote to
`Dynamic<T>` the day a second reader actually appears, not before.

### Activation-driven business state

A common variant: "when panel A activates, panel B should switch
to a particular view of the shared data." That data still lives in
app-shared state — the *trigger* for the switch is panel A's
`citizen_state.active`, which panel B reads reactively. Lifecycle
drives the transition; the data being viewed never moves into
`CitizenState`.

## Identities

```rust,ignore
pub struct CitizenId(pub String);
```

A `CitizenId` is a stable string identifier. The same id must be used
consistently across:

- `CitizenId::new("plot")` when constructing the panel.
- `dispatcher.register(CitizenId::new("plot"))` at startup.
- `dispatcher.activate(&CitizenId::new("plot"))` when the user
  clicks the corresponding tab.

If the strings disagree, the dispatcher silently treats them as
different citizens — `activate("plt")` will do nothing visible to a
panel registered as `"plot"`, and you'll burn an evening debugging
why a click does nothing.

Define ids as constants once and reference them everywhere:

```rust,ignore
const PLOT_ID:     &str = "plot";
const SETTINGS_ID: &str = "settings";

dispatcher.register(CitizenId::new(PLOT_ID));
dispatcher.register(CitizenId::new(SETTINGS_ID));

dispatcher.activate(&CitizenId::new(PLOT_ID));
```

This turns a typo into a compile error rather than a silent runtime
disconnect.

## When to override the hooks

The defaulted hooks just flip the corresponding `CitizenState` flag.
Override them when you need extra behavior alongside the flag flip:

```rust,ignore
impl Citizen for FetchPanel {
    // ... required methods ...

    fn on_activate(&mut self) {
        self.citizen_state_mut().active.set(true);
        self.start_background_fetch();         // app-specific
    }

    fn on_deactivate(&mut self) {
        self.citizen_state_mut().active.set(false);
        self.cancel_in_flight_fetch();
    }
}
```

In practice, most apps **do not** override the hooks. They let the
dispatcher do the flag flip and route side-effect logic through
[`CitizenMessage`](messages.md) instead — backend threads receive
`Activated { id: "fetch" }` and start the fetch from there. Override
the hooks only when the response is genuinely synchronous and
panel-local.

## How the trait is used at runtime

The `Citizen` trait is a **contract**, not a polymorphism mechanism:

- The dispatcher does *not* hold trait objects. It stores
  `CitizenState` clones in a `HashMap<CitizenId, CitizenState>`.
- Your `TabViewer` impl pulls panels by tab kind and calls each
  panel's own `show()` (or whatever you named it). The trait gives
  you uniform access to `id()` and `is_active()` if rendering needs
  it, but the dispatcher never walks an array of `dyn Citizen`.
- The hooks exist so panel code can call them on its own (e.g. from
  inside `show()` when a button is clicked), not because the
  dispatcher fires them.

The trait earns its keep by giving consistent shape across panels —
not by enabling runtime polymorphism over them.

## Summary

- Three required methods (`id`, `citizen_state`, `citizen_state_mut`),
  three defaulted hooks, two defaulted readers.
- Always obtain the `CitizenState` field from
  [`Dispatcher::register()`](dispatcher.md). Constructing it directly
  severs reactivity.
- Define citizen ids as `const`s so typos become compile errors.
- Override hooks only when the panel itself does synchronous extra
  work; otherwise route through [`CitizenMessage`](messages.md).
- The trait is a contract for shape, not a vehicle for runtime
  polymorphism.
