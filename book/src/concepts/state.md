# Reactive lifecycle: what `CitizenState` actually is

"Reactive lifecycle state" is jargon. Strip it apart:

- **Lifecycle** — the events a panel goes through during its time
  on screen: shown, hidden, activated, deactivated, clicked, moved.
- **Reactive** — when one of those facts changes, anyone reading it
  sees the new value automatically. No polling, no callback wiring.

Put together: `CitizenState` is a small bundle of lifecycle facts that
other code can read without asking "did this change since last time?"

## The fields

```rust
pub struct CitizenState {
    pub active: Dynamic<bool>,
    pub clicked: Dynamic<bool>,
    pub selected: Dynamic<bool>,
    pub moved: Dynamic<bool>,
    pub location: Dynamic<[f32; 2]>,
    pub visible: Dynamic<bool>,
}
```

| Field      | Meaning                                                            |
|------------|--------------------------------------------------------------------|
| `active`   | This citizen is the one currently active (the one-hot winner).     |
| `clicked`  | True for the frame this citizen was clicked.                       |
| `selected` | Persistent selection toggle, independent of activation.            |
| `moved`    | True if the citizen was moved to a new dock location.              |
| `location` | Last known position in the dock layout.                            |
| `visible`  | Whether the citizen is currently visible.                          |

Each field is a `Dynamic<T>` from `egui_mobius_reactive` — a handle into
shared reactive storage. It supports `.get()` and `.set()`, and the
underlying value lives behind an `Arc`.

## What "reactive" buys you

Imagine two panels: a **tab strip** and a **plot**. When the tab strip
activates the `freq_watt` citizen, the plot should redraw with frequency
versus watts.

The non-reactive version polls every frame:

```rust,ignore
// In the plot panel, every frame:
if app.current_tab != self.last_seen_tab {
    self.refresh();
    self.last_seen_tab = app.current_tab.clone();
}
```

You manually compare against a remembered value and act on the diff.
Every consumer that cares about "which tab is active" repeats this
dance. Each new consumer is another place to forget the comparison.

The reactive version just reads:

```rust,ignore
// In the plot panel, every frame:
if self.freq_watt_state.active.get() {
    // draw freq-watt data
}
```

`self.freq_watt_state` is a clone of the `CitizenState` the dispatcher
registered. The dispatcher writes `.active.set(true)` once when the
user clicks the tab; from that moment onward, every clone of that state
sees `true` on the next `.get()`. No diffing, no polling, no "last
seen" cache.

## Clones share storage — this is the whole game

```rust
use egui_citizen::CitizenState;

let state = CitizenState::new();
let clone = state.clone();

state.active.set(true);
assert!(clone.active.get()); // true
```

Each `Dynamic<T>` is, internally, an `Arc` over reactive storage.
Cloning a `CitizenState` clones the `Arc`s — both copies point at the
same underlying value. Set on one, see it on the other. **This is what
makes "reactive" work across panels and threads.**

A `CitizenState` is therefore not "owned" by anyone in particular. It's
a handle. The dispatcher holds one handle, your panel holds another,
and they refer to the same storage.

## The trap that bites everyone

`CitizenState::new()` and `CitizenState::default()` are public. They
look like ordinary constructors. They are not interchangeable with
"obtain a state from the dispatcher."

```rust,ignore
// WRONG — fresh storage, disconnected from the dispatcher
let state = CitizenState::new();
let panel = MyPanel::new(state);

dispatcher.activate(&CitizenId::new("my_panel"));

panel.citizen_state.active.get(); // still false!
```

Why: `dispatcher.activate()` writes to the `CitizenState` that *the
dispatcher itself owns*, registered at `register()` time. A separately
constructed `CitizenState` has its own fresh `Arc`s — the dispatcher
has no idea it exists, and the writes go somewhere else entirely.

The right way is always:

```rust,ignore
let state = dispatcher.register(CitizenId::new("my_panel"));
let panel = MyPanel::new(state);
```

`register()` builds a `CitizenState`, keeps one clone in the
dispatcher's table, and hands the other clone back to you. Both point
at the same storage. Now `dispatcher.activate()` and your panel see
the same value.

## When `CitizenState::new()` is fine

If a panel only displays its own citizen state, never reads from
another panel's, and never has another panel reading from it, and
activation isn't driving its UI — `CitizenState::default()` is
harmless. You just have a panel with its own private reactive bag of
bools.

In practice that case is rare. Default to `dispatcher.register()`.

## Summary

- `CitizenState` is six reactive `Dynamic<T>` fields covering the
  panel lifecycle: `active`, `clicked`, `selected`, `moved`,
  `location`, `visible`.
- "Reactive" means readers see writes immediately, with no polling
  and no callback wiring.
- Cloning a `CitizenState` shares storage. Constructing a fresh one
  does not.
- Always obtain a `CitizenState` from `dispatcher.register()` unless
  you are certain no one outside the panel reads or writes it.
