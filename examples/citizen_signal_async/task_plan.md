# Plan — citizen_signal_async

## What this example demonstrates

The intersection of the citizen pattern and `egui_mobius`'s
signal/slot bus, with a Tokio-async backend. Specifically: how a
citizen-pattern app gets *off the UI thread* without abandoning the
citizen abstraction, by using `Signal` / `AsyncDispatcher` as the
cross-thread bridge.

This is the "advanced" path foreshadowed in the introduction:

> `egui_mobius` Signals and Slots can be employed by the dispatcher
> for more advanced applications, in which case there is likely
> multiple background threads all receiving a signal from the
> dispatcher. Each background thread can then send a Signal back to
> a Slot on the dispatcher for task / thread completion.

## Panels

Three citizen panels in an `egui_dock` layout:

1. **Control** — a slider (work duration, ms) and a "Compute" button.
   Click sends a `WorkRequest` through the signal bus.
2. **Result** — displays the most recent `WorkResponse` value as a
   reactive `Dynamic<f64>`. Spinner while a request is in flight.
3. **Logger** — append-only scroll of backend events: request
   submitted, work started (timestamp), work completed, citizen
   activations.

## State

```rust,ignore
struct SharedState {
    work_duration_ms: Dynamic<u32>,    // Control writes
    last_result:      Dynamic<f64>,    // backend → UI
    in_flight:        Dynamic<bool>,   // backend → UI; Result panel renders spinner
    log:              Dynamic<Vec<String>>,  // any-side appends
}
```

Each panel holds a clone of `SharedState`. No cross-panel coupling
beyond the shared `Dynamic`s — Path A only between UI panels.

## Backend types

```rust,ignore
#[derive(Debug, Clone)]
struct WorkRequest {
    duration_ms: u32,
    seed: f64,
}

#[derive(Debug, Clone)]
struct WorkResponse {
    value: f64,           // computed result (sin of seed * 2π, say)
    elapsed_ms: u32,
}
```

Both are `Send + 'static` (required for the signal bus).

## Wiring (pseudocode)

```rust,ignore
// At app construction:
let citizen_dispatcher = CitizenDispatcher::new();
// Register three citizens, get back CitizenStates …

let async_dispatcher = AsyncDispatcher::<WorkRequest, WorkResponse>::new();
let (work_signal,   work_slot)   = factory::create_signal_slot::<WorkRequest>();
let (result_signal, result_slot) = factory::create_signal_slot::<WorkResponse>();

// Hook the backend up to the slot:
let log = state.log.clone();
async_dispatcher.attach_async(
    work_slot,
    result_signal.clone(),
    move |req| {
        let log = log.clone();
        async move {
            append(&log, format!("[backend] start, dur={}ms", req.duration_ms));
            tokio::time::sleep(Duration::from_millis(req.duration_ms as u64)).await;
            let value = (req.seed * 2.0 * std::f64::consts::PI).sin();
            append(&log, format!("[backend] done, value={value:.4}"));
            WorkResponse { value, elapsed_ms: req.duration_ms }
        }
    },
);

// UI-side: receive results
let last_result = state.last_result.clone();
let in_flight   = state.in_flight.clone();
let log         = state.log.clone();
result_slot.start(move |resp| {
    last_result.set(resp.value);
    in_flight.set(false);
    append(&log, format!("[ui] result received: {:.4}", resp.value));
});

// Control panel: on Compute click
work_signal.send(WorkRequest { duration_ms, seed }).unwrap();
state.in_flight.set(true);
```

## File layout

```text
examples/citizen_signal_async/
├── Cargo.toml
├── task_plan.md
├── task_findings.md
├── task_progress.md
└── src/
    ├── main.rs        # eframe::App, dock layout, drain loop
    ├── state.rs       # SharedState, WorkRequest, WorkResponse
    ├── tabs.rs        # TabKind, Tab, TabViewer
    ├── messages.rs    # AppMessage enum (Compute, …)
    ├── backend.rs     # async work function + signal/slot wiring helper
    └── panels/
        ├── mod.rs
        ├── control.rs
        ├── result.rs
        └── logger.rs
```

Mirrors `filter_plotter` deliberately — anything a reader has
already learned from the tutorial example transfers verbatim.
The new material is `backend.rs` and the additional wiring in
`main.rs`.

## Open questions / decisions to make

- **Tokio runtime ownership.** `AsyncDispatcher::new()` brings up its
  own runtime. Confirm there is no `#[tokio::main]` collision; the
  example must run with a plain `fn main()`.
- **Logger fan-out.** Two options for getting backend events into the
  Logger panel: (a) the backend's async closure appends directly to
  `log: Dynamic<Vec<String>>` (cheap, demonstrated above), (b) emit a
  separate `LogSignal` for backend events and have a third slot
  subscribed on the UI side. (b) better demonstrates fan-out via a
  second signal/slot pair; (a) is shorter. Default to (a) for the
  first cut, mention (b) in a comment.
- **Citizen activation hookup.** Tab clicks should call
  `citizen_dispatcher.activate(&id)` per the standard pattern.
  Drained `Activated` / `Deactivated` messages append to the log
  alongside backend events — proves both worlds compose in one
  Logger panel.
- **In-flight cancellation.** Out of scope for v1. Note this in the
  README at the end.

## Acceptance for v1

- `cargo run -p citizen_signal_async` opens the dock layout.
- Move the slider, click Compute. Result panel shows the new value
  after roughly the slider's duration. Logger shows three lines
  per click: `[backend] start`, `[backend] done`, `[ui] result`.
- Click a tab header. Logger shows `Activated { … }` for that
  citizen. Same `Deactivated { … }` for the previously-active one.
- No deadlocks if Compute is clicked rapidly; in-flight requests
  queue and complete in order.
