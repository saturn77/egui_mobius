# The Dispatcher

The `Dispatcher` is opt-in infrastructure, not the entry point of an
`egui_citizen` app. Many apps that share state between panels through
`Dynamic<T>` never reach for one. You reach for a dispatcher when the
reactive primitives don't already give you what you need:

- **One-hot activation arbitration.** Exactly one panel "active" at a
  time, atomically, across an arbitrary number of citizens. Useful
  any time focus, selection, or panel priority matters — even in a
  pure-UI app with no backend.
- **A queue for outbound events.** A place to push events that the
  update loop drains once per frame and forwards onward to backend
  threads, loggers, persistence, or anywhere else outside the UI tick.

Whichever of those you need, the dispatcher does three jobs:

1. **Owns the registered citizens' `CitizenState` handles.** Every
   citizen you call `register()` on has its `CitizenState` cloned into
   a `HashMap` inside the dispatcher. Panels hold the other clone;
   both refer to the same `Arc`-backed storage.
2. **Enforces the one-hot activation invariant.**
   `Dispatcher::activate(&id)` sets the named citizen's `active` flag
   to `true` and clears every other registered citizen's flag,
   atomically.
3. **Buffers outbound messages.** Lifecycle changes and explicit
   `send()` calls accumulate in a queue that you drain once per frame.

The dispatcher does *not* know about dock layout, does *not* fire UI
events on its own, and does *not* observe arbitrary `Dynamic<T>`
writes (see
[the coupling chapter](coupling.md#the-dispatcher-is-not-a-reactive-bus)).

![Six panels (Project, Settings, Plotter 1, Plotter 2, Logger, Terminal/Shell) in a 2×2 dock layout. Each panel contains a labelled state cloud — ProjectState, SettingsState, Plotter1State, Plotter2State, LoggerState, TerminalState. Arrows from every state cloud converge on a single DISPATCHER block on the right.](../images/Basic_App_State.drawio.png)

*Registration topology. Every panel hands its `CitizenState` to the
one dispatcher; the dispatcher keeps a clone in its table while the
panel keeps the other clone. Both refer to the same `Arc`-backed
storage, which is exactly why activations issued from the dispatcher
become immediately visible to every panel that holds a clone.*

## API surface

```rust,ignore
pub struct Dispatcher { /* private */ }

impl Dispatcher {
    pub fn new() -> Self;
    pub fn register(&mut self, id: CitizenId) -> CitizenState;
    pub fn get(&self, id: &CitizenId) -> Option<&CitizenState>;
    pub fn send(&mut self, message: CitizenMessage);
    pub fn activate(&mut self, id: &CitizenId);
    pub fn drain_messages(&mut self) -> Vec<CitizenMessage>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

Eight methods. The four that matter day-to-day are `register`,
`activate`, `drain_messages`, and `send`.

## `register(id) -> CitizenState`

```rust,ignore
let plot_state = dispatcher.register(CitizenId::new("plot"));
let plot       = PlotPanel::new(plot_state);
```

Registers a citizen and hands you back a `CitizenState` *handle* into
the dispatcher's storage. The dispatcher keeps a clone of that handle
in its table; you take the other clone and pass it to your panel
struct. Both point at the same underlying `Arc<Mutex<...>>` storage,
so writes by either side are immediately visible to the other (see
[reactive lifecycle: clones share storage](state.md#clones-share-storage--this-is-the-whole-game)).

This is the **only** correct way to obtain a `CitizenState` for a
panel that the dispatcher will activate. `CitizenState::default()`
allocates fresh disconnected storage; the dispatcher and the panel
end up holding different `Arc`s and the reactive link is silently
severed.

## `activate(&id)`

```rust,ignore
dispatcher.activate(&CitizenId::new("plot"));
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
for msg in dispatcher.drain_messages() {
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
dispatcher.send(CitizenMessage::Clicked { id: alpha_id.clone() });
```

Pushes a message onto the queue without going through `activate()`.
Common uses:

- **Custom lifecycle events.** Emit
  [`Selected`, `Moved`, `VisibilityChanged`](messages.md) from
  app-level code that detects them — the dispatcher only emits
  `Activated`/`Deactivated` on its own.
- **App-message bridging.** A backend thread that needs to inject a
  citizen-shaped event back into the UI loop.
- **Testing / replay.** Replaying a recorded session from a log file.

The dispatcher does no validation here — it accepts any
`CitizenMessage` you give it.

## Worked example

```rust,ignore
use egui_citizen::{CitizenId, CitizenMessage, Dispatcher};

let mut dispatcher = Dispatcher::new();

let alpha = dispatcher.register(CitizenId::new("alpha"));
let beta  = dispatcher.register(CitizenId::new("beta"));

dispatcher.activate(&CitizenId::new("alpha"));
let msgs = dispatcher.drain_messages();
// msgs == [Activated { id: alpha }]
//                                    (beta was never active, so no Deactivated)

dispatcher.activate(&CitizenId::new("beta"));
let msgs = dispatcher.drain_messages();
// msgs == [Activated { id: beta }, Deactivated { id: alpha }]

assert!(beta.active.get());
assert!(!alpha.active.get());
```

## One dispatcher per app

The one-hot invariant is **per-dispatcher**. Two dispatchers in the
same app each maintain their own one-hot, and `activate` on one does
not deactivate citizens registered on the other. If two halves of
your UI ever race over "who is active," you have two dispatchers (or
worse, two `CitizenState`s for the same logical citizen).

The dispatcher typically lives on the app struct:

```rust,ignore
struct App {
    dispatcher: Dispatcher,
    plot:       PlotPanel,
    settings:   SettingsPanel,
    /* ... */
}
```

Backend threads that need to send messages do so via a
`crossbeam_channel` whose receiver lives on the UI thread; the UI
thread drains the channel and forwards messages with
`dispatcher.send()`. The `Dispatcher` itself is `&mut self`-only and
is not shared across threads directly.

## Summary

- `register()` is the only correct way to obtain a panel's
  `CitizenState`.
- `activate()` is an encoded set/reset; call it on user-driven events,
  never every frame unconditionally.
- `drain_messages()` is the once-per-frame backend boundary; forgetting
  to call it leaks memory.
- `send()` is for explicit Path B messages outside the activation
  flow.
- One dispatcher per app — never two.
