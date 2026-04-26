# The Citizen trait

The `Citizen` trait is how a panel struct *becomes* a citizen. It
gives the panel three things:

1. A persistent identity — a [`CitizenId`](#identities).
2. A handle to its [reactive lifecycle state](state.md) — a
   `CitizenState`.
3. Default lifecycle hooks (`on_activate`, `on_deactivate`,
   `on_click`) and reader convenience methods (`is_active`,
   `is_selected`).

This chapter covers the trait surface. For *how to use* it across a
real app — including which panel-author state goes where — see
[Where does state live?](../patterns/state-shape.md).

## The trait

```rust,ignore
pub trait Citizen {
    fn id(&self) -> &CitizenId;
    fn citizen_state(&self) -> &CitizenState;
    fn citizen_state_mut(&mut self) -> &mut CitizenState;

    // Defaulted hooks (override if you need custom behavior):
    fn on_activate(&mut self)   { self.citizen_state_mut().active.set(true); }
    fn on_deactivate(&mut self) { self.citizen_state_mut().active.set(false); }
    fn on_click(&mut self)      { self.citizen_state_mut().clicked.set(true); }

    // Defaulted readers:
    fn is_active(&self)   -> bool { self.citizen_state().active.get() }
    fn is_selected(&self) -> bool { self.citizen_state().selected.get() }
}
```

Three required methods. Three defaulted hooks. Two defaulted readers.
That is the whole trait.

## Minimum-viable impl

```rust,ignore
use egui_citizen::{Citizen, CitizenId, CitizenState};

struct PlotPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
}

impl PlotPanel {
    fn new(state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new("plot"),
            citizen_state: state,
        }
    }
}

impl Citizen for PlotPanel {
    fn id(&self) -> &CitizenId              { &self.citizen_id }
    fn citizen_state(&self) -> &CitizenState { &self.citizen_state }
    fn citizen_state_mut(&mut self) -> &mut CitizenState {
        &mut self.citizen_state
    }
}
```

Five lines of trait impl, all delegating to fields. That's the
ceremony to wire a panel into the citizen system.

The `state` argument to `PlotPanel::new` should always come from
[`Dispatcher::register()`](dispatcher.md#registerid---citizenstate),
**never** from `CitizenState::new()` or `CitizenState::default()`. The
latter allocate fresh disconnected storage and silently sever the
reactive link with the dispatcher (see
[the trap in the state chapter](state.md#the-trap-that-bites-everyone)).

## Identities

```rust,ignore
pub struct CitizenId(pub String);
```

A `CitizenId` is a stable string identifier. The same id must be used
consistently across:

- `CitizenId::new("plot")` when constructing the panel.
- `dispatcher.register(CitizenId::new("plot"))` at startup.
- `dispatcher.activate(&CitizenId::new("plot"))` when the user
  clicks the corresponding tab.

If the strings disagree, the dispatcher silently treats them as
different citizens — `activate("plt")` will do nothing visible to a
panel registered as `"plot"`, and you'll burn an evening debugging
why a click does nothing.

Define ids as constants once and reference them everywhere:

```rust,ignore
const PLOT_ID:     &str = "plot";
const SETTINGS_ID: &str = "settings";

dispatcher.register(CitizenId::new(PLOT_ID));
dispatcher.register(CitizenId::new(SETTINGS_ID));

dispatcher.activate(&CitizenId::new(PLOT_ID));
```

This turns a typo into a compile error rather than a silent runtime
disconnect.

## When to override the hooks

The defaulted hooks just flip the corresponding `CitizenState` flag.
Override them when you need extra behavior alongside the flag flip:

```rust,ignore
impl Citizen for FetchPanel {
    // ... required methods ...

    fn on_activate(&mut self) {
        self.citizen_state_mut().active.set(true);
        self.start_background_fetch();         // app-specific
    }

    fn on_deactivate(&mut self) {
        self.citizen_state_mut().active.set(false);
        self.cancel_in_flight_fetch();
    }
}
```

In practice, most apps **do not** override the hooks. They let the
dispatcher do the flag flip and route side-effect logic through
[`CitizenMessage`](messages.md) instead — backend threads receive
`Activated { id: "fetch" }` and start the fetch from there. Override
the hooks only when the response is genuinely synchronous and
panel-local.

## How the trait is used at runtime

The `Citizen` trait is a **contract**, not a polymorphism mechanism:

- The dispatcher does *not* hold trait objects. It stores
  `CitizenState` clones in a `HashMap<CitizenId, CitizenState>`.
- Your `TabViewer` impl pulls panels by tab kind and calls each
  panel's own `show()` (or whatever you named it). The trait gives
  you uniform access to `id()` and `is_active()` if rendering needs
  it, but the dispatcher never walks an array of `dyn Citizen`.
- The hooks exist so panel code can call them on its own (e.g. from
  inside `show()` when a button is clicked), not because the
  dispatcher fires them.

The trait earns its keep by giving consistent shape across panels —
not by enabling runtime polymorphism over them.

## Summary

- Three required methods (`id`, `citizen_state`, `citizen_state_mut`),
  three defaulted hooks, two defaulted readers.
- Always obtain the `CitizenState` field from
  [`Dispatcher::register()`](dispatcher.md). Constructing it directly
  severs reactivity.
- Define citizen ids as `const`s so typos become compile errors.
- Override hooks only when the panel itself does synchronous extra
  work; otherwise route through [`CitizenMessage`](messages.md).
- The trait is a contract for shape, not a vehicle for runtime
  polymorphism.
