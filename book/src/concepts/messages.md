# CitizenMessage — the backend bridge

`CitizenMessage` is the discriminated lifecycle event the dispatcher
emits and your code consumes. It is the data payload of
[Path B](coupling.md#path-b--dispatcher-messages-panel-to-backend) —
the UI-to-backend coupling channel.

## The variants

```rust,ignore
pub enum CitizenMessage {
    Activated         { id: CitizenId },
    Deactivated       { id: CitizenId },
    Clicked           { id: CitizenId },
    Selected          { id: CitizenId, selected: bool },
    Moved             { id: CitizenId, location: [f32; 2] },
    VisibilityChanged { id: CitizenId, visible: bool },
}
```

| Variant              | Fired by                                                | Payload                       |
|----------------------|---------------------------------------------------------|-------------------------------|
| `Activated`          | `Dispatcher::activate(&id)`                             | id that became active         |
| `Deactivated`        | `Dispatcher::activate(&id)` for previously-active citizens | id that lost active        |
| `Clicked`            | App code (via `Dispatcher::send`)                       | id that was clicked           |
| `Selected`           | App code (selection toggling)                           | id + new selection state      |
| `Moved`              | App code (after a dock-layout move)                     | id + new `[x, y]` location    |
| `VisibilityChanged`  | App code (after a tab is shown / hidden)                | id + new visibility           |

Note the asymmetry. `Activated` and `Deactivated` are produced
**automatically** by `Dispatcher::activate()`. The other four exist
so that app code can route the corresponding lifecycle facts through
the same queue, but you must push them yourself via
[`Dispatcher::send()`](dispatcher.md#sendmessage).

`CitizenMessage` derives `Clone` and `Debug`. It does *not* derive
`PartialEq` — if you need to compare messages, match on the variants
explicitly.

## Identity: `CitizenId`

```rust,ignore
pub struct CitizenId(pub String);
```

The id is a stable string. Same identity rules as in
[the Citizen trait chapter](citizen.md#identities) — define them as
`const`s and pass through `CitizenId::new(...)` consistently.

## Consuming messages

The canonical loop:

```rust,ignore
fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    DockArea::new(&mut self.tabs).show(ctx, &mut self.tab_viewer);

    for msg in self.dispatcher.drain_messages() {
        match msg {
            CitizenMessage::Activated { id } => {
                self.log.push(format!("[{id}] activated"));
            }
            CitizenMessage::Deactivated { id } => {
                self.log.push(format!("[{id}] deactivated"));
            }
            _ => {}
        }
    }
}
```

Drain once per frame, **after** `DockArea::show()` has had a chance
to fire `on_tab_button` (and therefore `dispatcher.activate()`). If
you drain *before* `show()`, you'll see one frame of latency on every
activation — the message produced this frame won't be observed until
next frame's drain.

## Forwarding to a backend thread

The typical Path B shape: the UI drains the dispatcher and forwards
each message into a channel that a backend thread is reading.

```rust,ignore
use crossbeam_channel::{unbounded, Sender};

// At startup:
let (tx, rx) = unbounded::<CitizenMessage>();
std::thread::spawn(move || {
    for msg in rx {
        match msg {
            CitizenMessage::Activated { id } if id.0 == "fetch" => {
                start_http_request();
            }
            CitizenMessage::Deactivated { id } if id.0 == "fetch" => {
                cancel_in_flight();
            }
            _ => {}
        }
    }
});

// In update():
for msg in self.dispatcher.drain_messages() {
    let _ = tx.send(msg.clone());   // forward
    /* ... and process locally if needed ... */
}
```

The `_ = tx.send(...)` discards "receiver disconnected" errors,
which can happen if the backend thread has exited. The backend
thread's `match` decides what each message means in *its* domain —
the dispatcher doesn't care.

Do not invoke egui or wgpu state from the backend thread. If the
backend needs to surface results back to the UI, send them through a
return channel that the UI thread drains in its update loop, or
write them into a `Dynamic<T>` that the relevant panel observes via
[Path A](coupling.md#path-a--shared-dynamict-panel-to-panel).

## Wrapping in your own app message enum

For non-trivial apps, you will have many messages that are not
lifecycle events — file open and close, view manipulations,
computed-result notifications, hotkey actions, background-task
completions. The idiomatic pattern is to wrap `CitizenMessage`
inside your own `AppMessage` enum, then run *all* app-level events
through that one queue.

A realistic shape, drawn from a real-world app — `CopperForge`, a
KiCad PCB tool with twelve dockable panels:

```rust,ignore
use std::path::PathBuf;
use egui_citizen::CitizenMessage;

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// A citizen lifecycle event (activated, deactivated, etc.)
    Citizen(CitizenMessage),

    // ── Project ─────────────────────────────────────────────
    ProjectLoaded { path: PathBuf },
    ProjectClosed,
    PcbFileSelected { path: PathBuf },

    // ── Layers ──────────────────────────────────────────────
    LayersReloaded,
    LayerVisibilityChanged { layer_name: String, visible: bool },

    // ── View ────────────────────────────────────────────────
    ResetView,
    FlipBoard,
    Rotate { degrees: f32 },

    // ── DRC ─────────────────────────────────────────────────
    DrcRunRequested,
    DrcCompleted { violation_count: usize },

    // ── Hotkeys ─────────────────────────────────────────────
    HotkeyPressed(Hotkey),
}

#[derive(Debug, Clone)]
pub enum Hotkey {
    Flip,
    Rotate,
    ToggleUnits,
    /* ... */
}
```

Citizen lifecycle is one variant out of a dozen-plus. That ratio is
typical: in any non-trivial app, lifecycle events are a *minority*
of message traffic, and the dispatcher's drain point becomes the
single funnel for *all* app-level events. Three patterns make the
shape legible at scale.

### Section dividers via comments

The `// ── Project ──...─` separators turn a variant-heavy enum into
something a reader can scan in one pass. Group by domain, dividers
between groups. `rustfmt` leaves comment lines alone, so the layout
survives formatting passes. Purely a legibility tool — but it earns
its place quickly as the enum grows.

### Intent and outcome both flow as messages

Notice the pairing of `DrcRunRequested` with `DrcCompleted`. Both
travel through the same queue. The intent variant ("the user pressed
Run DRC") is what triggers the work; the outcome variant ("DRC
finished, here are the results") is what consumers react to.

This is the Elm-style discipline at work: **the message loop is a
temporal sequence of events, not a function-call graph**. The
drained log reads back as a record of what happened in order —
inspectable, loggable, replayable. Resist the temptation to call a
function directly when the user clicks "Run DRC"; emit a
`DrcRunRequested` message and let the drain loop dispatch it to the
worker thread.

The pairing pattern generalizes:

| Intent variant            | Outcome variant     |
|---------------------------|---------------------|
| `DrcRunRequested`         | `DrcCompleted`      |
| `ProjectOpenRequested`    | `ProjectLoaded`     |
| `BomRebuildRequested`     | `BomUpdated`        |

Not every action needs both ends. Some are pure intent (`ResetView`)
because there is no meaningful "completed" state. Some are pure
outcome (`LayersReloaded`, fired by a file-watcher) because there is
no UI-side intent. Use the pair when there is asynchronous work
between the two and consumers care about both endpoints.

### Cancellation rides on the lifecycle queue

Long-running async work raises a third question alongside intent and
outcome: **what cancels in-flight work when the user moves on?** The
answer is already in the queue — it's the `Deactivated { id }` message
that [`Dispatcher::activate()`](dispatcher.md#activateid) produces
automatically whenever the previously-active panel loses its slot.

The pattern, drawn from the same forwarding loop above:

```rust,ignore
// Backend thread, consuming the channel:
for msg in rx {
    match msg {
        CitizenMessage::Activated   { id } if id.0 == "fetch" => start_fetch(),
        CitizenMessage::Deactivated { id } if id.0 == "fetch" => cancel_in_flight(),
        _ => {}
    }
}
```

The dispatcher does not return a work handle from `send()`. It cannot —
it doesn't own any work. The backend thread that *does* own the work
also owns its own in-flight state (a `JoinHandle`, a `CancellationToken`,
an `AbortHandle`, whatever the runtime provides) and reacts to the
deactivation message by aborting that work itself.

For synchronous, panel-local cleanup that runs on the UI thread,
override [`Citizen::on_deactivate`](citizen.md#when-to-override-the-hooks)
alongside the flag flip:

```rust,ignore
fn on_deactivate(&mut self) {
    self.citizen_state_mut().active.set(false);
    self.flush_local_buffers();   // synchronous, on the UI thread
}
```

For *asynchronous* cancellation — anything that involves stopping a
thread, aborting a task, or interrupting IO — route it through the
message queue, never through the override. The override runs on the UI
thread and must not block.

### The long-running work pattern, end to end

Putting intent, outcome, and cancellation together yields a predictable
shape for any unit of async work in a citizen app:

1. **Intent message** — a user-initiated event lands in the dispatcher
   queue (`DrcRunRequested`, `ProjectOpenRequested`) and gets drained
   into the backend channel.
2. **Backend dispatch** — the backend thread spawns the work and
   remembers its in-flight handle (`JoinHandle`, `AbortHandle`, etc.)
   keyed by the originating panel's `CitizenId`.
3. **Cancellation lifecycle** — if the user activates a different panel,
   `Deactivated { id }` arrives on the same channel; the backend thread
   aborts the remembered handle for that id.
4. **Outcome message** — when work completes (or is cancelled), the
   backend pushes an outcome message back to the UI thread through a
   return channel, drained into the next frame's `update()`.

Every step is a message. Every message flows through the same queue (or
a paired return channel). The UI thread holds no futures, no join
handles, no cancellation tokens — only `Dynamic<T>` cells that observe
the results when outcome messages land.

This is the Elm discipline applied to a Rust GUI: a temporal sequence of
events, fully inspectable, never a tangled call graph. The app can be
paused, logged, and replayed by recording the message stream alone.

### Sub-domain nesting

`HotkeyPressed(Hotkey)` is a single top-level variant that wraps a
separate `Hotkey` enum. The flat alternative —
`HotkeyFlipPressed`, `HotkeyRotatePressed`,
`HotkeyToggleUnitsPressed`, … — bloats the top-level enum and forces
hotkey-handling code to match on a sprawling pattern instead of a
focused sub-enum.

Use sub-domain nesting whenever a domain has its own internal
vocabulary that is likely to grow. Keyboard hotkeys, view controls,
file operations, background-task progress reports, vendor-specific
release packaging — all of these are good candidates for their own
sub-enum nested under one outer variant.

### Putting it together: the drain loop

The dispatcher's queue still carries only `CitizenMessage`. The app
wraps each citizen message as it drains, and non-citizen variants
are produced by app code emitting them directly through the same
backend channel:

```rust,ignore
fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
    DockArea::new(&mut self.tabs).show(ctx, &mut self.tab_viewer);

    // Drain citizen messages, wrap as AppMessage, forward.
    for msg in self.dispatcher.drain_messages() {
        let app_msg = AppMessage::Citizen(msg);
        self.event_log.push(app_msg.clone());
        let _ = self.tx_backend.send(app_msg);
    }

    // App code emits its own non-citizen variants through the same
    // channel — e.g. when a worker thread reports DRC results back:
    if let Some(result) = self.drc_worker.try_recv() {
        let _ = self.tx_backend.send(
            AppMessage::DrcCompleted { violation_count: result.len() },
        );
    }
}
```

One queue, many message families, one drain pass per frame.
`CopperForge` runs this exact shape across twelve dockable panels.

## What `CitizenMessage` is *not*

It is **not a general-purpose event bus.** The dispatcher does not
provide subscriptions, filtering, prioritization, or replay. If you
need those, build them on top — typically in your `AppMessage` layer
or as a separate logger / event-store.

It is **not — emphatically — a replacement for shared `Dynamic<T>`
state.** UI-to-UI coupling should still go through
[Path A (shared `Dynamic<T>`)](coupling.md#path-a--shared-dynamict-panel-to-panel).
Messages are reserved for genuine event signals (*things happened*)
rather than continuous state updates (*things are*). A panel that
needs to mirror another panel's slider value should clone the
slider's `Dynamic<f32>` and read it; it should not subscribe to a
`SliderChanged` message stream.

## Summary

- Six variants, all carrying at least a `CitizenId`.
- `Activated` and `Deactivated` are emitted automatically by
  `Dispatcher::activate()`. The other four require explicit
  `dispatcher.send(...)`.
- Drain once per frame, after `DockArea::show()`.
- Forward to backend threads via `crossbeam_channel`. Don't touch
  egui from the consumer side.
- For app-level events beyond lifecycle, wrap `CitizenMessage` in
  your own `AppMessage` enum.
- Reserve messages for events. Use [`Dynamic<T>`](state.md) for
  state.
