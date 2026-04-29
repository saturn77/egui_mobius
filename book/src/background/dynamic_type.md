

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

### Where it sits in the wider crate

`egui_mobius_reactive` also provides `Value<T>` (an older API with the
same idea), `Derived<T>` (computed values that recalculate when their
inputs change), and a `SignalRegistry` for app-wide signal wiring.
`egui_citizen` uses only `Dynamic<T>`, so that is all this book
covers. If you later want a `Derived<T>` that reads a `CitizenState`
field and recomputes downstream, the reactive crate's own documentation
is the next stop.

The chapter on [reactive lifecycle](concepts/state.md) builds on this
foundation and walks through the trap that bites users who construct a
`CitizenState` with `CitizenState::default()` instead of obtaining one
from `Dispatcher::register()`.
