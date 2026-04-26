# Inside `Dynamic<T>`

The [introduction](../introduction.md#the-dynamict-primitive) covers
what `Dynamic<T>` looks like from outside: a thread-safe cell with
`get`, `set`, `lock`, and `on_change`. This chapter opens the box.

You don't *need* this material to use `egui_citizen` — Path A
(shared-state polling, see [Coupling paths](coupling.md)) only needs
the high-level API. But you *do* need it the moment you reach for
`on_change`, profile reactive overhead, or wonder why two seemingly
identical-looking subscriptions behave differently. Read this chapter
before you write code that subscribes to a `Dynamic<T>`.

## The struct, peeled outside-in

```rust,ignore
pub struct Dynamic<T> {
    inner: Arc<Mutex<T>>,
    notifiers: Arc<parking_lot::Mutex<Vec<Sender<()>>>>,
}
```

Two `Arc`s. The first holds the value; the second holds the
notification machinery. The intro already covered the value side. Here
we open the second one.

### `Arc<...>`

Same role as on `inner`: every clone of a `Dynamic<T>` must refer to
the **same** notifier list. If clone A registers a subscriber via
`on_change` and clone B calls `.set(...)`, B's set must wake A's
subscriber — otherwise the whole "shared reactive cell" story
collapses. Cloning the outer `Dynamic<T>` clones this `Arc`, so all
clones share one list.

### `parking_lot::Mutex<...>`

Note this is **not** `std::sync::Mutex`. The value side uses
`std::sync::Mutex<T>`; the notifier side uses `parking_lot::Mutex`.
Two reasons that matter in practice:

1. **Lower overhead.** `parking_lot::Mutex` is one word, no poisoning
   bookkeeping, faster on the uncontended path. The notifier lock is
   touched on *every* `.set()` call and on *every* `on_change()` call,
   so the constant factor adds up.
2. **No poison ceremony.** `std::sync::Mutex::lock()` returns
   `LockResult<...>`, which is `Err` if a previous holder panicked.
   `parking_lot::Mutex::lock()` returns the guard directly. The
   notifier path doesn't want to thread `unwrap()` through every
   iteration.

For the value side, `std::sync::Mutex` is fine because access is
mediated by `get`/`set`/`lock` and the cost is negligible relative to
whatever the user is doing with the value.

### `Vec<Sender<()>>`

A growable list of channel senders. **One sender per subscriber.**
Each call to `on_change(cb)` creates a fresh mpsc channel `(tx, rx)`,
pushes `tx` into this vec, and spawns a thread that blocks on
`rx.recv()`. The vec accumulates these senders as subscribers
register over the program's lifetime.

### `Sender<()>`

`std::sync::mpsc::Sender<()>`. The payload is unit. The channel is a
pure **doorbell** — "the value changed, wake up." It carries no
information about *what* changed, because the subscriber already
captured the `Dynamic<T>` in its closure and can call `.get()`
directly to read the current value.

This is also why every subscriber's channel has the same payload
type: they're all signaled the same way, regardless of what `T` is on
the parent `Dynamic`.

## What `set()` actually does

```rust,ignore
pub fn set(&self, value: T) {
    let mut guard = self.inner.lock().unwrap();
    *guard = value;
    // (inner mutex drops here)
    for notifier in self.notifiers.lock().iter() {
        let _ = notifier.send(()); // ignore closed-channel errors
    }
}
```

Two locks taken, sequentially:

1. Lock the value mutex, write the new value, drop the lock.
2. Lock the notifier mutex, iterate, fan out one wakeup per sender,
   drop the lock.

The two locks never overlap, so writers don't hold the value mutex
while iterating subscribers — important for keeping `get()` calls on
other threads from blocking unnecessarily.

Sends are **fire-and-forget**: if a receiver thread has died, the
`send` returns `Err`, and the `let _ =` discards it. The dead sender
stays in the vec, but no panic and no bubbled error.

## What `on_change` actually does

```rust,ignore
fn on_change<F>(&self, callback: F) -> Arc<F>
where
    F: Fn() + Send + Sync + 'static,
{
    let cb = Arc::new(callback);
    let cb_clone = cb.clone();
    let (tx, rx) = channel();
    self.notifiers.lock().push(tx);
    thread::spawn(move || {
        while rx.recv().is_ok() {
            cb_clone();
        }
    });
    cb
}
```

Three allocations and one OS thread per subscriber:

1. `Arc<F>` wrapping the callback — returned to the caller, also held
   by the worker thread.
2. An mpsc channel — `tx` lives in the notifier vec, `rx` lives on
   the worker thread's stack.
3. `thread::spawn` — a dedicated, non-pooled OS thread that loops on
   `rx.recv()` until the channel closes.

The worker thread is *not* shared. Subscribe to 50 `Dynamic`s and you
spawn 50 threads.

## Putting the producer and consumer together

```text
┌─────────────────────┐                     ┌──────────────────────┐
│ Dynamic<T>          │                     │ subscriber thread    │
│                     │                     │ (one per on_change)  │
│  inner    ◀─writes──│  caller does set()  │                      │
│  notifiers ┐        │                     │                      │
│            │        │                     │                      │
│            └─[tx]───┼──── mpsc channel ───┼───[rx] rx.recv() loop│
│                     │      send(())       │     calls callback() │
└─────────────────────┘                     └──────────────────────┘
```

`set()` writes the value and rings every doorbell in the notifier
list. Each subscriber's worker thread wakes up, runs its callback,
and goes back to waiting. Dead simple, deliberately so.

## Practical implications

These follow directly from the implementation. They are the reasons
`egui_citizen` recommends Path A polling over `on_change` for most
in-app reactivity:

### Subscriptions cost an OS thread each

Cheap for a handful, less cheap for hundreds. The threads are
dedicated, not pooled. For dense reactive UI inside an egui app,
prefer reading `.get()` once per frame in `ui()` (Path A) over
spawning a worker thread per `Dynamic`.

### There is no `unsubscribe` API

The `Vec<Sender<()>>` only grows. Once you call `on_change`, the
sender lives in the notifier list until the `Dynamic<T>` itself is
dropped — which happens when the *last* outer `Arc` is released, and
the dispatcher and panels typically hold those for the program's
lifetime.

Dropping the returned `Arc<F>` does **not** tear down the worker
thread. The thread waits on `rx.recv()`, which only returns `Err`
when the *sender* side is dropped — and the sender is in the notifier
vec, where nothing removes it.

If you need teardown, you must wrap the `Dynamic<T>` itself in a
container whose `Drop` releases all `Arc` references — i.e., teardown
is at the granularity of the entire reactive cell, not the individual
subscription.

### No coalescing

`mpsc::channel()` is unbounded. Rapid `.set()` calls enqueue one
wakeup each, and the worker thread runs the callback once per
wakeup. If you `set` 100 times in quick succession (e.g., dragging a
slider), the worker runs the callback 100 times.

The callback can read `.get()` and observe whatever the latest value
is at that moment, but the wakeups themselves do not merge. Some
reactive systems coalesce ("one redraw per microtask"); this one does
not. If coalescing matters to you, debounce in your callback or in
the consumer of whatever channel your callback feeds.

### Wakeups run off the UI thread

The worker is a vanilla `thread::spawn`. **Do not touch egui or wgpu
state from inside an `on_change` callback.** The egui context is not
`Send`/`Sync` in the way you'd need for that.

The legitimate jobs for a callback:

- Push to a `crossbeam_channel::Sender` that the UI thread drains in
  its update loop.
- Trigger a `Dispatcher::send(...)` if you have access to a
  thread-safe wrapper around it.
- Do off-thread work (write to a log file, fire an HTTP request,
  recompute something heavy and stash the result for the UI to pick
  up).

### `set()` holds the notifier lock while iterating

Concurrent `on_change` calls block until the iteration completes.
Concurrent `set()` calls also serialize through the notifier lock.

In practice, the notifier vec is small (a handful of subscribers per
`Dynamic` in real apps) and the iteration is fast (each `send(())` is
a constant-time channel operation). It is not a hotspot. But it is a
bottleneck if you imagine pathological cases — thousands of
subscribers, microsecond-scale sets, many threads. Don't build
those.

## When to use what

Three coupling tools, three different jobs:

| Tool                  | Best for                                         | Cost                          |
|-----------------------|--------------------------------------------------|-------------------------------|
| `.get()` in `ui()`    | UI-to-UI state sharing                           | One atomic read per frame     |
| `dispatcher.send`     | UI-to-backend events with one drain point        | One queue push, drained once  |
| `Dynamic::on_change`  | Off-thread reactions independent of the UI loop  | One OS thread per subscriber  |

The default in `egui_citizen` is the top row. Reach for the bottom
row only when something genuinely needs to react *outside* the UI's
frame cycle — and even then, prefer routing through the dispatcher
rather than spawning per-`Dynamic` worker threads, since the
dispatcher gives you one drain point instead of N callbacks.

## Summary

```text
Arc<parking_lot::Mutex<Vec<Sender<()>>>>
└── shared, mutex-protected, growable list of mpsc senders, each
    representing one subscriber's "doorbell" line
```

`set()` rings every doorbell in the vec. Each subscriber's worker
thread wakes up and runs its callback. That is the entire mechanism.

The simplicity of this notification subsystem is what makes
`Dynamic<T>` cheap and predictable for the dominant use case
(panel-side polling). It is also why callback-style subscriptions —
while supported — are best used sparingly, off the UI thread, and
ideally via the dispatcher rather than directly.
