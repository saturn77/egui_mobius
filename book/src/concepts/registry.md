# The Registry

The `Registry` is a **registry of citizens**. Each citizen registers
a `CitizenState` handle with it, and the registry coordinates one-hot
activation and message buffering across the registered set. If you've
built backend systems, this is the same pattern you've seen before — a
central table that knows what's plugged in and routes accordingly. The
[citizens-as-plug-ins framing](../background/what_is_a_citizen.md#citizens-as-plug-ins)
is the other side of this contract: plug-ins register, the registry
coordinates, no party needs direct references to the others.

It is also opt-in infrastructure, not the entry point of an
`egui_citizen` app. Many apps that share state between panels through
`Dynamic<T>` never reach for one. You reach for a registry when the
reactive primitives don't already give you what you need:

- **One-hot activation arbitration.** Exactly one panel "active" at a
  time, atomically, across an arbitrary number of citizens. Useful
  any time focus, selection, or panel priority matters — even in a
  pure-UI app with no backend.
- **A queue for outbound events.** A place to push events that the
  update loop drains once per frame and forwards onward to backend
  threads, loggers, persistence, or anywhere else outside the UI tick.

Whichever of those you need, the registry does three jobs:

1. **Owns the registered citizens' `CitizenState` handles.** Every
   citizen you add via `add().with_name()` has its `CitizenState` cloned into
   a `HashMap` inside the registry. Panels hold the other clone;
   both refer to the same `Arc`-backed storage.
2. **Enforces the one-hot activation invariant.**
   `Registry::activate(name)` sets the named citizen's `active` flag
   to `true` and clears every other registered citizen's flag,
   atomically.
3. **Buffers outbound messages.** Lifecycle changes and explicit
   `send()` calls accumulate in a queue that you drain once per frame.

## Why the registry, not just shared state?

If panels can share state through `Dynamic<T>` clones already, what
does the registry add?

Three things that don't fall out of shared `Dynamic<T>` alone:

- **Atomic one-hot activation.** `activate(name)` flips one citizen's
  `active` to `true` and clears every other registered citizen's flag
  in a single call. Doing this with shared state alone means wiring
  each panel to clear every other panel's flag — N² coordination and
  a new wire every time a panel is added.
- **Lookup by stable id.** Panels are addressed by `CitizenId`, not
  by pointers to one another's structs, and the registry is the
  UI-side directory for that lookup. (Backend threads never query
  it — the registry stays on the UI thread; a backend gets its
  `Dynamic<T>` clones handed to it at spawn.)
- **Frame-aligned event buffering.** `activate()` and `send()` push
  lifecycle events into a queue that `drain_messages()` consumes once
  per frame. This is the seam between event-time (a tab was clicked
  just now) and frame-time (work reacts to it next tick) — backend
  threads see batched updates rather than per-mutation callbacks.

And three things it does *not* do, each a common confusion:

- **It is not a runtime gateway.** Buffering events is not the same as
  performing the work those events trigger. The registry does not
  spawn threads, schedule async tasks, or own `JoinHandle`s. Its queue
  is the *interface* to the runtime boundary; the boundary itself is
  the backend thread your app starts and feeds from
  [`drain_messages()`](#drain_messages---veccitizenmessage).
- **It is not a reactive bus.** The registry tracks `CitizenState`
  (six lifecycle fields) and nothing else. Your slider's
  `Dynamic<f32>` is invisible to the registry until you explicitly
  call [`send()`](#sendmessage). See
  [the coupling chapter](coupling.md#the-registry-is-not-a-reactive-bus)
  for why that asymmetry is deliberate.
- **It does not observe dock layout or fire UI events on its own.** A
  tab click triggers `activate(...)` because *your* `on_tab_button`
  calls it; the registry never spies on the dock and fires
  activations for you. Give it an event, it routes; don't give it an
  event, it sits idle.

![Six panels (Project, Settings, Plotter 1, Plotter 2, Logger, Terminal/Shell) in a 2×2 dock layout. Each panel contains a labelled state cloud — ProjectState, SettingsState, Plotter1State, Plotter2State, LoggerState, TerminalState. Arrows from every state cloud converge on a single DISPATCHER block on the right.](../images/Basic_App_State.drawio.png)

*Registration topology. Every panel hands its `CitizenState` to the
one registry; the registry keeps a clone in its table while the
panel keeps the other clone. Both refer to the same `Arc`-backed
storage, which is exactly why activations issued from the registry
become immediately visible to every panel that holds a clone.*

## API surface

```rust,ignore
pub struct Registry { /* private */ }

impl Registry {
    pub fn new() -> Self;
    pub fn add(&mut self) -> CitizenBuilder<'_>;
    pub fn get(&self, id: impl Into<CitizenId>) -> Option<&CitizenState>;
    pub fn send(&mut self, message: CitizenMessage);
    pub fn activate(&mut self, id: impl Into<CitizenId>);
    pub fn drain_messages(&mut self) -> Vec<CitizenMessage>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

pub struct CitizenBuilder<'a> { /* private */ }

impl CitizenBuilder<'_> {
    pub fn with_name(self, name: impl Into<CitizenId>) -> CitizenState;
}
```

Eight methods. The four that matter day-to-day are `add`,
`activate`, `drain_messages`, and `send`. Anywhere an id is
expected, a plain `"name"` works — `CitizenId` converts from
`&str`, `String`, and `&CitizenId`.

## `add().with_name(name)`

```rust,ignore
let plot_state = registry.add().with_name("plot");
let plot       = PlotPanel::new(plot_state);
```

Adds a citizen and hands you back a `CitizenState` *handle* into
the registry's storage. The registry keeps a clone of that handle
in its table; you take the other clone and pass it to your panel
struct. Both point at the same underlying `Arc<Mutex<...>>` storage,
so writes by either side are immediately visible to the other (see
[reactive lifecycle: clones share storage](state.md#clones-share-storage--this-is-the-whole-game)).

This is the **only** correct way to obtain a `CitizenState` for a
panel that the registry will activate. `CitizenState::default()`
allocates fresh disconnected storage; the registry and the panel
end up holding different `Arc`s and the reactive link is silently
severed.

`add()` returns a builder so per-citizen options have somewhere to
live as they grow — they chain between `add()` and the
`with_name()` finisher.

## `activate(name)`

```rust,ignore
registry.activate("plot");
```

The encoded set/reset. After this call:

- The named citizen's `active` flag is `true`.
- Every *other* registered citizen's `active` flag is `false`.
- The message queue contains an `Activated { id }` for the named
  citizen — **always**, even if the citizen was already active in
  the previous frame.
- The queue contains `Deactivated { id }` for each citizen that was
  *previously* active and has now been turned off. (Citizens that
  were already off do not produce a `Deactivated`.)

Call `activate()` from a user-driven event. For `egui_dock` apps,
that's `TabViewer::on_tab_button` when `response.clicked()`. **Do
not** call it from the render path unconditionally — that fires
`Activated` (and possibly `Deactivated`) every frame and floods the
queue. See [pitfalls](../pitfalls.md).

## `drain_messages() -> Vec<CitizenMessage>`

```rust,ignore
for msg in registry.drain_messages() {
    match msg {
        CitizenMessage::Activated { id }   => { /* ... */ }
        CitizenMessage::Deactivated { id } => { /* ... */ }
        _ => {}
    }
}
```

Removes and returns all pending messages. Call once per frame, after
`DockArea::show()` (so that `on_tab_button` has had its chance to
call `activate()`). The queue is now empty until the next `activate`
or `send` produces more.

If you forget to drain, the queue grows forever. No errors, no
warnings — just a slow memory leak.

## `send(message)`

```rust,ignore
registry.send(CitizenMessage::Clicked { id: alpha_id.clone() });
```

Pushes a message onto the queue without going through `activate()`.
Common uses:

- **Custom lifecycle events.** Emit
  [`Selected`, `Moved`, `VisibilityChanged`](messages.md) from
  app-level code that detects them — the registry only emits
  `Activated`/`Deactivated` on its own.
- **App-message bridging.** A backend thread that needs to inject a
  citizen-shaped event back into the UI loop.
- **Testing / replay.** Replaying a recorded session from a log file.

The registry does no validation here — it accepts any
`CitizenMessage` you give it.

## Worked example

```rust,ignore
use egui_citizen::{CitizenId, CitizenMessage, Registry};

let mut registry = Registry::new();

let alpha = registry.add().with_name("alpha");
let beta  = registry.add().with_name("beta");

registry.activate("alpha");
let msgs = registry.drain_messages();
// msgs == [Activated { id: alpha }]
//                                    (beta was never active, so no Deactivated)

registry.activate("beta");
let msgs = registry.drain_messages();
// msgs == [Activated { id: beta }, Deactivated { id: alpha }]

assert!(beta.active.get());
assert!(!alpha.active.get());
```

## One registry per app

The one-hot invariant is **per-registry**. Two registries in the
same app each maintain their own one-hot, and `activate` on one does
not deactivate citizens registered on the other. If two halves of
your UI ever race over "who is active," you have two registries (or
worse, two `CitizenState`s for the same logical citizen).

The registry typically lives on the app struct:

```rust,ignore
struct App {
    registry: Registry,
    plot:       PlotPanel,
    settings:   SettingsPanel,
    /* ... */
}
```

Backend threads that need to send messages do so via a
`crossbeam_channel` whose receiver lives on the UI thread; the UI
thread drains the channel and forwards messages with
`registry.send()`. The `Registry` itself is `&mut self`-only and
is not shared across threads directly.

## Summary

- `add().with_name()` is the only correct way to obtain a panel's
  `CitizenState`.
- `activate()` is an encoded set/reset; call it on user-driven events,
  never every frame unconditionally.
- `drain_messages()` is the once-per-frame backend boundary; forgetting
  to call it leaks memory.
- `send()` is for explicit Path B messages outside the activation
  flow.
- One registry per app — never two.
