# Writing a mobius_lang citizen plugin

A **plugin** lets a `.mobius` file instantiate a real, compiled `egui_mobius`
citizen — `egui_lens`, `egui_quill`, `egui_plot`, or one you wrote — by name.
The language places it and binds its ports; the plugin is the small piece of
Rust that receives the DSL-wired `Dynamic<T>` handles and constructs the real
crate.

This is the framework's plugin architecture: the host binary declares a
vocabulary of citizens, and any `.mobius` file composes them. Nothing in the
language knows about any particular citizen — it only knows names and ports.

## The two halves

```
.mobius source                     host binary (Rust)
──────────────                     ──────────────────
let plot = PlotPanel(       ◄────  Plugins::new().register("PlotPanel", PlotPanel::build)
    bench = bench.display)
                                   PlotPanel::build(name, bindings) -> Box<dyn CitizenView>
```

The registry name in source (`PlotPanel`) must match the name the host
registers. Lowering rejects any instantiation of a name the host didn't
provide, with a source span — so a typo is a compile-time error, not a blank
panel.

## A plugin is three things

Every plugin is a struct that implements two traits and offers one
constructor. Here is the complete `egui_lens` logger plugin from
`examples/mobius_lang_demo/src/plugins.rs`:

### 1. Identity + lifecycle — `egui_citizen::Citizen`

The struct holds a `CitizenId` and a `CitizenState`, exactly like any
hand-written citizen:

```rust
pub struct LensLogger {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    log_state: Dynamic<ReactiveEventLoggerState>, // the real egui_lens state
    colors: Dynamic<LogColors>,
    amplitude: Option<Watch<f32>>,                // observed DSL values
    trace: Option<Watch<Vec<f32>>>,
    last_amplitude: f32,
    last_trace_len: Option<usize>,
}

impl Citizen for LensLogger {
    fn id(&self) -> &CitizenId { &self.citizen_id }
    fn citizen_state(&self) -> &CitizenState { &self.citizen_state }
    fn citizen_state_mut(&mut self) -> &mut CitizenState { &mut self.citizen_state }
}
```

### 2. Per-frame draw — `mobius_lang_host::CitizenView`

`Citizen` is lifecycle-only (and object-safe). `CitizenView` adds the one
method a dock host needs — the draw:

```rust
impl CitizenView for LensLogger {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.log_changes(); // react to observed values
        ReactiveEventLogger::with_colors(&self.log_state, &self.colors).show(ui);
    }
}
```

That `.show(ui)` is the real `egui_lens` widget — the same call CopperMine
makes. The DSL host renders it identically because it *is* the same crate.

### 3. The constructor — `build(name, bindings)`

This is where the typed doorway is crossed. The `BindingSet` holds this
instance's wired ports as type-erased handles; the plugin downcasts exactly
the ones it needs with `read::<T>()` (observe-only, an `in` port),
`write::<T>()` (writable, an `out` port), or `emit()` (a signal). A
wrong-typed or absent port returns `None` rather than lying:

```rust
impl LensLogger {
    pub fn build(name: &str, bindings: &BindingSet) -> Box<dyn CitizenView> {
        let log_state = Dynamic::new(ReactiveEventLoggerState::new());
        let colors = Dynamic::new(LogColors::default());
        Box::new(Self {
            citizen_id: CitizenId::new(name.to_owned()),
            citizen_state: CitizenState::new(),
            log_state,
            colors,
            amplitude: bindings.read::<f32>("bench.amplitude"), // in f32
            trace: bindings.read::<Vec<f32>>("bench.trace"),    // in Vec<f32>
            last_amplitude: f32::NAN,
            last_trace_len: None,
        })
    }
}
```

The `Watch<T>` returned by `read` exposes `get()` (and `on_change` for
`T: PartialEq`) but no `set` — the language's `in`/`out` direction becomes a
Rust type. That is how the one-writer discipline survives into the running
program.

## Registering the plugin

The host builds a `Plugins` table and feeds its names to the language
registry, so validation and construction share one vocabulary:

```rust
fn plugins() -> Plugins {
    Plugins::new()
        .register("PlotPanel",  PlotPanel::build)
        .register("LensLogger", LensLogger::build)
}

// ... during elaboration:
let mut registry = Registry::new().handler("bench_worker").event("BenchCmd");
for name in plugins.names() {
    registry = registry.citizen(name);
}
let ir = lower(app, &registry)?;      // validates instantiations
let wired = wire(&ir, &Host::new())?; // constructs Dynamic<T> values

// ... per instance:
let view: Option<Box<dyn CitizenView>> = plugins.build(&instance); // None = source citizen
```

## The `.mobius` side

The file instantiates the plugin by name and binds its ports to shared
values through a modport view:

```
let plot = PlotPanel(bench = bench.display);   // display = (in amplitude, in enabled, in trace)
let log  = LensLogger(bench = bench.display);
```

`bench.display` is the observe-only view, so both plugins receive read
handles — matching the `read::<T>()` calls in their constructors. If you bind
a view whose directions don't match what the plugin reads or writes, the
mismatch surfaces at lowering with a span, not at runtime.

## Custom value types

`f32`, `bool`, `String`, and `Vec<f32>` are built into the `Host`. If a
citizen shares a domain type — a `Scene`, a `Config` — register it so the
language can declare and default it:

```rust
let host = Host::new().value_type::<Scene, _>("Scene", |literal| parse_scene(literal));
```

Then `scene : Scene` is a legal shared value in source, and
`bindings.write::<Scene>("app.scene")` hands the plugin the real
`Dynamic<Scene>`.

## Checklist for a new plugin

1. A struct holding `CitizenId` + `CitizenState` + whatever handles/state it
   needs.
2. `impl Citizen` (three one-line accessors).
3. `impl CitizenView` with `ui(&mut self, ui)` that draws the real crate.
4. A `build(name, bindings) -> Box<dyn CitizenView>` that `read`/`write`/`emit`s
   its ports.
5. `register("TypeName", MyCitizen::build)` in the host's `Plugins`.
6. Register any custom value types on the `Host`.

That is the whole contract. A `.mobius` file can now compose your citizen
alongside every other, wire shared state between them, and route events —
all as diffable text.
