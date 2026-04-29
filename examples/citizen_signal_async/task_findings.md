# Findings — egui_mobius signal/slot + AsyncDispatcher + egui_citizen

Research notes gathered before writing `citizen_signal_async`. The
goal of this file is to anchor the design in real APIs that exist in
the workspace today, not in invented ones.

## The two `Dispatcher`s are unrelated

`egui_mobius` and `egui_citizen` each export a type named `Dispatcher`,
and they have nothing to do with each other:

- `egui_citizen::Dispatcher` — panel lifecycle. `register()` /
  `activate()` / `drain_messages()`. Single-threaded, UI-side.
- `egui_mobius::Dispatcher<E>` — synchronous signal-slot bus.
  `register_slot(channel, fn)` / `send(channel, event)`.
- `egui_mobius::dispatching::AsyncDispatcher<E, R>` — the same idea
  but spins up a Tokio runtime and runs slots as async tasks.

This example uses `egui_citizen::Dispatcher` for panel lifecycle and
`egui_mobius::dispatching::AsyncDispatcher` for backend work. Both
must be in scope at once. Convention used in existing examples:
qualified paths (`use egui_citizen::Dispatcher as CitizenDispatcher`
or fully-qualified `egui_mobius::dispatching::AsyncDispatcher`).

## Signal / Slot

From `crates/egui_mobius/src/`:

```rust,ignore
// signals.rs
pub struct Signal<T> { /* sender side */ }
impl<T> Signal<T> {
    pub fn send(&self, msg: T) -> Result<(), String>;
    pub fn send_multiple(&self, vec: Vec<T>) -> Result<(), String>;
}

// slot.rs
pub struct Slot<T> { /* receiver side */ }
impl<T: Send + 'static> Slot<T> {
    pub fn start<F>(&mut self, handler: F)
        where F: FnMut(T) + Send + 'static;
    pub fn start_async<F, Fut>(&mut self, handler: F);
}

// factory.rs
pub fn create_signal_slot<T>() -> (Signal<T>, Slot<T>);
```

`Signal<T>` is cheap to clone and `Send` — backend threads hold one
to push back to the UI. `Slot<T>::start` consumes the receiver and
spawns its own thread to drive the handler. **Each `(Signal, Slot)`
pair is point-to-point, not multicast.** Fan-out to multiple
consumers means either multiple signal/slot pairs, or a single slot
whose handler does all the consumer work in its closure.

## AsyncDispatcher

From `crates/egui_mobius/src/dispatching.rs`:

```rust,ignore
pub struct AsyncDispatcher<E, R> { /* internals */ }
impl<E, R> AsyncDispatcher<E, R> {
    pub fn new() -> Self;        // brings up a tokio runtime
    pub fn attach_async<F, Fut>(
        &self,
        slot: Slot<E>,
        signal: Signal<R>,
        handler: F,
    ) where F: Fn(E) -> Fut + Send + Sync + 'static,
           Fut: Future<Output = R> + Send + 'static;
}
```

`attach_async` is the bridge: a `Slot<E>` (UI-thread input) becomes
the input to an async handler whose return value is pushed back via
`Signal<R>`. The handler runs on the dispatcher's Tokio runtime, so
it can `.await` freely.

## Wiring pattern — taken from `examples/clock_async/src/main.rs`

Verbatim, from lines 155–174:

```rust,ignore
let (event_signal, event_slot) = factory::create_signal_slot::<Event>();
let (response_signal, response_slot) = factory::create_signal_slot::<Response>();

let dispatcher = AsyncDispatcher::new();
dispatcher.attach_async(
    event_slot,
    response_signal.clone(),
    |event: Event| async move {
        match event {
            Event::SliderChanged(val) => {
                tokio::time::sleep(Duration::from_millis(300)).await;
                Response::SliderProcessed(val)
            }
            // ...
        }
    },
);
```

UI side, lines 24–42, uses `response_slot.start(|resp| { ... })` to
receive results on the UI thread and write to shared state.

Backend-thread send-back, lines 107–119: a separate worker thread
holds a `Signal<ClockMessage>` and calls `.send(...)` on it; the UI
thread has the matching slot.

## egui_citizen integration surface

`egui_citizen::Dispatcher` does not know about signals at all. The
bridge between the two worlds happens in the UI update loop:

1. UI thread calls `citizen_dispatcher.drain_messages()` once per
   frame and routes any `CitizenMessage` it cares about through to
   `signal.send(...)`.
2. Backend thread emits results via `signal.send(response)`; the UI
   thread's matching `Slot::start(...)` handler writes the response
   into `Dynamic<T>` cells (Path A) and/or appends a log line.

In other words: `egui_citizen::Dispatcher` is the entry point on the
UI side; `Signal` / `AsyncDispatcher` are the cross-thread bus
beyond that. There is no library-level glue — the glue is two
function calls in `App::update`.

## Required imports for the example

```rust,ignore
use egui_citizen::{
    Dispatcher as CitizenDispatcher, CitizenId, CitizenMessage, CitizenState, Citizen,
};
use egui_mobius::{Signal, Slot};
use egui_mobius::factory;
use egui_mobius::dispatching::AsyncDispatcher;
use egui_mobius_reactive::Dynamic;
```

## Gotchas to remember

- **`Slot::start` consumes the receiver.** You can't have two slots
  reading the same channel; for fan-out, emit on multiple signals.
- **`AsyncDispatcher::new()` spins up a Tokio runtime.** Don't also
  call `#[tokio::main]` on `main` — single runtime per process.
- **Panel `Dispatcher` clones share storage** (`egui_citizen` rule)
  but `Signal<T>` clones share *channel*, not value. Different
  semantics; don't mentally collapse them.
- **One-frame latency from backend → UI.** The slot handler runs on
  the slot's worker thread, writes to a `Dynamic<T>`; egui sees the
  new value on the next frame. Same single-frame delay as Path B.
