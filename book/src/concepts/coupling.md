# Coupling paths: UI-to-UI and UI-to-backend

`egui_citizen` gives you two distinct ways for a state change in one
place to influence work somewhere else. They serve different jobs,
they have different timing, and a single widget — what we'll call an
**atom** (a widget inside a citizen panel) — can use either path or
both at once.

Get this vocabulary clear before you design anything substantial.

## The two paths

| Path | Mechanism           | Good for                    | Timing           |
|------|---------------------|-----------------------------|------------------|
| A    | Shared `Dynamic<T>` | Panel → panel               | Immediate        |
| B    | `dispatcher.send()` | Panel → backend / logger    | Next drain pass  |

### Path A — shared `Dynamic<T>` (panel to panel)

Two panels share a clone of the same `Dynamic<T>`. One writes, the
other reads. That's the whole mechanism.

```rust,ignore
// settings_panel.rs
struct SettingsPanelState {
    pub slider_value: Dynamic<f32>,
}

// logger_panel.rs
struct LoggerPanelState {
    pub observed_slider: Dynamic<f32>, // clone of settings.slider_value
}
```

The two `Dynamic<f32>` handles point at the same `Arc<Mutex<f32>>`.
When `settings` calls `.set(...)`, the write lands in shared memory.
When `logger.ui()` runs on the next frame and calls `.get()`, it sees
the new value.

No subscription, no callback, no event bus. egui already redraws
frequently enough that polling-per-frame is effectively free. **Path A
carries state, not events.**

### Path B — dispatcher messages (panel to backend)

The settings panel explicitly enqueues a message; the app's update
loop drains it after `DockArea::show()` and forwards it onward — to a
backend thread, a logger sink, a persistence layer.

```rust,ignore
// In settings_panel.rs, when the slider changes:
dispatcher.send(AppMessage::SliderChanged(local));

// In App::update(), once per frame:
for msg in dispatcher.drain_messages() {
    match msg {
        AppMessage::SliderChanged(v) => tx_backend.send(v).unwrap(),
        // ...
    }
}
```

**Path B carries events, not state.** Each `.send()` enqueues one
record; `drain_messages()` consumes the queue. Nothing is "shared" —
the message is a value, not a handle.

> **Aside on `Dynamic::on_change`.** `egui_mobius_reactive` does
> provide a callback-style subscription on `Dynamic<T>` itself, which
> looks superficially like a third coupling option. It is — but it
> spawns one OS thread per subscriber, has no unsubscribe API, and
> doesn't coalesce wakeups. For most `egui_citizen` apps, Path B
> through the dispatcher is the better way to do "react off the UI
> thread when this value changes." The full mechanics live in
> [Inside `Dynamic<T>`](inside-dynamic.md).

## Atoms can wire to both

A single atom can fan out to both paths from the same user event. The
fan-out happens at the change handler:

```rust,ignore
if ui.add(Slider::new(&mut local, 0.0..=100.0)).changed() {
    self.slider_value.set(local);                       // Path A
    dispatcher.send(AppMessage::SliderChanged(local));  // Path B
}
```

This is a common and correct pattern. The atom is the **single write
site**; each path is a derived consequence of the one event. Readers
on Path A see the new value next frame; consumers on Path B get the
message on the next drain cycle.

### When dual wiring is right

Dual-wire an atom when a change needs to:

- Update shared UI state, **and**
- Trigger side-effect work (log it, persist it, send it to a backend
  thread, recompute a derived value off-thread).

Single-path atoms are fine when the change only needs one of those.
Don't dual-wire out of habit — extra messages with no consumers are
noise.

## Source-of-truth discipline

The trap: once an atom writes to both paths, downstream code can read
*from either one* and the two representations can drift. Pick a
discipline and hold it.

**Discipline 1 — `Dynamic<T>` is canonical, the message is a ping.**

```rust,ignore
dispatcher.send(AppMessage::SliderChanged);  // no value in the message!
// Consumers re-read `settings.slider_value.get()` when the message arrives.
```

Consumers of the message reach back into shared state for the current
value. This guarantees consistency — there is exactly one value, the
one in the `Dynamic<f32>`. The message says only "something happened,
go look."

**Discipline 2 — Message carries the value, `Dynamic<T>` is a UI mirror.**

```rust,ignore
dispatcher.send(AppMessage::SliderChanged(local));
```

The message is the canonical record of the event. The `Dynamic<f32>`
exists only so other panels can render the current value without
intercepting messages. Consumers of the message trust the message and
do not re-read the `Dynamic`.

Either discipline works. **Mixing them silently** — some consumers
trusting the message, others reading the `Dynamic` — is where bugs
live.

## Timing

Within a single frame, the two paths do not tick in lockstep:

- **Path A is instant.** `.set(v)` returns after writing; any panel
  calling `.get()` on a clone sees `v` immediately.
- **Path B is queued.** `.send(msg)` appends to the dispatcher's queue;
  consumers don't see it until the update loop calls
  `drain_messages()`.

Backend threads therefore observe the Path B message *after* the UI
has already observed the Path A value. For typical use (the backend
does work and replies asynchronously), that one-frame gap is
invisible. For anything tighter — a dependency on in-frame ordering
between UI and backend — you need to redesign, not lean harder on
this.

## The dispatcher is *not* a reactive bus

Worth saying explicitly, because it is the mistake everyone makes
coming from other reactive systems:

> The dispatcher does not observe `Dynamic<T>` writes.

It only knows about the `CitizenState` fields it registered, and its
queue only fills from `activate()` and explicit `send()`. Your
slider's `Dynamic<f32>` on `SettingsPanelState` is invisible to it
until the panel explicitly bridges the two paths with a `.send()`
call.

Shared state (Path A) and dispatcher messages (Path B) **compose** —
they don't **chain** automatically.

## Summary

- Two coupling paths. **Path A** for UI-to-UI state sharing (shared
  `Dynamic<T>`). **Path B** for UI-to-backend events
  (`dispatcher.send()` + `drain_messages()`).
- Atoms can wire to one path or both. Fan-out happens at the write
  site, not downstream.
- Dual-wired atoms require a source-of-truth discipline: `Dynamic`
  canonical with the message as a ping, or message canonical with the
  `Dynamic` as a UI mirror. Don't mix.
- Path A is in-frame; Path B lands at the next drain. Backend threads
  see changes one frame after the UI does.
- The dispatcher is a lifecycle registry plus an explicit outbound
  queue. It is not an automatic observer of reactive state.
