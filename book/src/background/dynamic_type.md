

## The `Dynamic<T>` primitive

Before going further, know the one building block that everything
reactive in `egui_citizen` rests on:

> **Every reactive field in `egui_citizen` is a `Dynamic<T>`.**

### What it is

A `Dynamic<T>` is a thread-safe, observable container for a single
value. Internally (quoting `egui_mobius_reactive` verbatim):

```rust,ignore
pub struct Dynamic<T> {
    inner: Arc<Mutex<T>>,
    notifiers: Arc<parking_lot::Mutex<Vec<Sender<()>>>>,
}
```

Two `Arc`s. The first holds the value behind a standard `Mutex`. The
second holds a list of channel senders used to wake subscribers when
the value changes. `Dynamic<T>` derives `Clone` — cloning bumps the
refcount on each `Arc` and copies nothing else, so every clone refers
to the **same** storage and the **same** notifier list.

> **Aside: `Clone` in Rust is per-type.** Rust has no language-level
> default of "deep" vs. "shallow" — each type's `Clone` impl decides
> what cloning means for that type.
>
> - Owned types like `String`, `Vec<T>`, `Box<T>`, `HashMap<K, V>`
>   duplicate their heap data on `.clone()` — what C++ would call a
>   deep copy.
> - Reference-counted types like `Arc<T>` and `Rc<T>` are *documented*
>   to clone as a refcount increment — a new handle pointing at the
>   same allocation. C++ would call this shallow.
>
> `Mutex<T>` itself does not implement `Clone` at all — that is why
> the `Arc` wrapper is necessary. Cloning a `Dynamic<T>` is therefore
> exactly two `Arc::clone` calls: two refcount bumps, zero data
> duplication. The shared storage is not an accident; it is the
> precise contract of `Arc`.

### Core API

```rust,ignore
use egui_mobius_reactive::Dynamic;

let counter = Dynamic::new(0);      // construct
let n = counter.get();              // read (clones T out of the lock)
counter.set(42);                    // write, then notify listeners
let mut guard = counter.lock();     // direct MutexGuard if you need it
*guard += 1;
```

- `Dynamic::new(initial)` — requires `T: Clone + Send + 'static`.
- `get()` — returns a *clone* of the value; the lock is released
  before you work with the result.
- `set(value)` — takes the lock, writes, drops the lock, then sends
  `()` into every registered notifier channel.
- `lock()` — gives you a raw `MutexGuard` for in-place mutation. Other
  readers and writers block until you drop the guard.

### Observing changes

Reading on every frame works — that's what UI panels do via `.get()`
in their render methods. For event-driven work, `ValueExt::on_change`
registers a callback that fires on every mutation:

```rust,ignore
use egui_mobius_reactive::{Dynamic, ValueExt};

let counter = Dynamic::new(0);
counter.on_change(|| println!("changed!"));

counter.set(1); // prints "changed!"
counter.set(2); // prints "changed!"
```

Under the hood, `on_change` spawns a dedicated background thread that
waits on the notifier channel. The callback runs off the UI thread —
which is why `T` needs `Send + Sync + PartialEq + 'static` for this
path. The full mechanics — including what it actually costs per
subscriber and why the canonical reactive path inside `egui_citizen`
is panel-side polling rather than callbacks — get a chapter of their
own: [Inside `Dynamic<T>`](concepts/inside-dynamic.md).

### Why this shape matters for `egui_citizen`

Because clones share storage, a `CitizenState` — a bundle of
`Dynamic<T>` fields — is a **handle**, not an owned value:

```rust
use egui_citizen::CitizenState;

let a = CitizenState::new();
let b = a.clone();

a.active.set(true);
assert!(b.active.get());  // true — same Arc<Mutex<bool>>
```

The dispatcher keeps one clone of each citizen's state; your panel
holds another. When the dispatcher writes `.active.set(true)`, your
panel sees `true` on its next `.get()`. No event bus, no subscription
to wire up, no polling loop — just a shared `Arc`.

### Permissive type, disciplined use

`Dynamic<T>` itself is **multi-producer, multi-consumer** — any clone
can call `.set()`, any clone can `.get()`. The type doesn't restrict
who writes.

`egui_citizen` layers a **single-writer-per-field** discipline on top:

| Field                         | Canonical writer                  |
|-------------------------------|-----------------------------------|
| `active`                      | The `Dispatcher` (via `activate`) |
| `clicked`                     | The panel's `on_click` hook       |
| `selected`, `visible`, `moved`| The panel or app-level code       |
| `location`                    | The dock-integration layer        |

Readers are unrestricted: any panel, any backend thread. Writers are
by convention, not enforcement. This is why the dispatcher is central
(it's the one place that serializes activation writes across all
citizens), and why the [pitfall on two dispatchers in one
app](pitfalls.md) exists — two writers to the same logical field break
the one-hot invariant.

Keep to "one writer per field" and the reactive story stays clean;
violate it and you're back to the per-frame race the crate was built
to avoid.

### `ValueExt` and `Derived<T>`

`egui_citizen` itself only reaches for `Dynamic<T>`, but the reactive
crate ships two related building blocks that you'll want when an app
outgrows pure `Dynamic` storage. They both ride on the same notifier
infrastructure described above (the shared `Vec<Sender<()>>`), so
understanding them is mostly a matter of seeing what each one wraps.

#### `ValueExt` — the `on_change` extension trait

`ValueExt<T>` is a tiny extension trait whose only method is
`on_change`:

```rust,ignore
pub trait ValueExt<T: Clone + Send + Sync + 'static> {
    fn on_change<F>(&self, callback: F) -> Arc<F>
    where
        F: Fn() + Send + Sync + 'static;
}
```

It's implemented for `Dynamic<T>` (when `T: PartialEq`) and is the
public surface for callback-style subscription. Importing it brings
`.on_change(...)` into scope:

```rust,ignore
use egui_mobius_reactive::{Dynamic, ValueExt};

let counter = Dynamic::new(0);
counter.on_change(|| println!("changed"));
```

Without `ValueExt` in scope, calling `.on_change` on a `Dynamic`
won't compile — that's the whole reason the trait exists. Every
example earlier in this chapter that uses `on_change` is implicitly
relying on `ValueExt` being imported.

The cost of each `on_change` registration — one OS thread per
subscriber, no unsubscribe — is detailed in the [Inside
`Dynamic<T>`](../concepts/inside-dynamic.md) chapter. Within
`egui_citizen` itself, panel-side `.get()` polling is the canonical
path for UI reactivity; `on_change` is for off-thread reactions
(file logging, network sends, anything outside the egui frame loop).

#### `Derived<T>` — auto-recomputed values

A `Derived<T>` is a read-only reactive cell whose value is *computed
from* one or more `Dynamic`s (or other `Derived`s). It recomputes
automatically whenever one of its inputs changes:

```rust,ignore
use egui_mobius_reactive::{Dynamic, Derived};
use std::sync::Arc;

let count = Dynamic::new(0);
let count_arc = Arc::new(count.clone());
let doubled = Derived::new(&[count_arc.clone()], move || {
    count_arc.get() * 2
});

count.set(5);
// `doubled.get()` now returns 10 — the closure re-ran when count changed.
```

Two facts make `Derived<T>` cheap and predictable:

1. **`get()` is a clone of a cached value, not a recomputation.** The
   closure runs only when an input changes, not every time you read
   the result. Reading `doubled.get()` 60 times per frame is just
   60 lock-acquire-and-clone operations on a `Mutex<T>`.

2. **Inputs subscribe to the closure via the same notifier
   plumbing.** `Derived::new(deps, compute)` calls
   `dep.subscribe(...)` on each dependency, which pushes a
   `Sender<()>` into the dependency's notifier vec — the same vec
   that backs `ValueExt::on_change`. When a dep's `set()` rings the
   doorbell, the `Derived` re-runs its closure and stores the new
   value in its own cache.

Practically, this means `Derived<T>` is the right tool when you
have a value that **must always agree with** other reactive state —
"the formatted version of `current_time`," "the filtered subset of
`logs`," "the sum of two `Dynamic<i32>`s." Use a `Derived` and the
arithmetic stays correct without anyone remembering to call an
update function.

Three honest caveats:

- The closure re-runs once per `set()` on any dependency. If a
  panel writes its dependency 100 times during a slider drag, the
  closure runs 100 times. No coalescing.
- The closure takes ownership of all captured state, so the
  dependency you read inside is typically a separate clone bound
  via the closure (the `count_arc` in the example above).
- Cycles aren't detected. `Derived` A depending on `Derived` B
  depending on `Derived` A will recurse and panic. Don't.

#### Together

`ValueExt::on_change` and `Derived::new` are two consumers of the
same primitive. The notifier vec inside `Dynamic<T>` is just a list
of "things to wake when this value changes" — `on_change` adds one
that runs your closure on a worker thread, `Derived::new` adds one
that re-runs the compute closure and caches the result. Same hook,
different jobs. This is also the reason both APIs cost an entry in
that vec and neither has an unsubscribe — they're pinned for the
`Dynamic`'s lifetime.

`egui_mobius_reactive` also provides `Value<T>` (an older API with
the same shape as `Dynamic<T>`) and a `SignalRegistry` for app-wide
signal wiring. The book doesn't cover those — the reactive crate's
own documentation is the next stop.

### Where this leads next

The chapter on [reactive lifecycle](../concepts/state.md) builds on
this foundation and walks through the trap that bites users who
construct a `CitizenState` with `CitizenState::default()` instead of
obtaining one from `Dispatcher::register()`. The [Inside
`Dynamic<T>`](../concepts/inside-dynamic.md) chapter opens up the
notifier mechanism in detail — read it before writing code that
subscribes to a `Dynamic<T>`.
