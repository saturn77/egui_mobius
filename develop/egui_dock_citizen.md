# egui_dock + Citizen — toward individuated breakaway panels

Design notes for treating dockable panels as stateful Citizens, and the
path from there to **breakaway panels** — a docked panel dragged out to
become its own OS window, Qt-Ads style.

Captures a design discussion from the egui Discord `#egui_dock` channel,
grounded in the real `egui_citizen` types.

## Thesis: a dock layout is geometry, not architecture

`egui_dock` is excellent at one question — *where does a panel live* —
splits, tabs, drag-to-rearrange. Most dockable-panel libraries stop
there and leave the rest to the application. But a real application
needs two more answers:

- **What is a panel** — its identity and its state.
- **How do panels talk** — to each other, and to a backend.

egui_mobius answers all three with a clean division of labor:

| Question | Owner |
|----------|-------|
| *Where* a panel is shown | `egui_dock` |
| *What* a panel is | `Citizen` + `CitizenState` |
| *How* panels coordinate | `Dispatcher` |

The dock tree is geometry. The Citizen and the Dispatcher are the
architecture. A dockable-panel system without that backbone is
incomplete for enterprise work — it can arrange panels but can't make
them stateful, observable, or coordinated.

## The Citizen pattern

A panel becomes a `Citizen` by implementing a small trait: a stable
identity, reactive lifecycle state, and defaulted lifecycle hooks.

```rust
pub trait Citizen {
    fn id(&self) -> &CitizenId;
    fn citizen_state(&self) -> &CitizenState;
    fn citizen_state_mut(&mut self) -> &mut CitizenState;

    // Defaulted hooks — override for custom behavior.
    fn on_activate(&mut self)   { self.citizen_state_mut().active.set(true); }
    fn on_deactivate(&mut self) { self.citizen_state_mut().active.set(false); }
    fn on_click(&mut self)      { self.citizen_state_mut().clicked.set(true); }

    // Defaulted readers.
    fn is_active(&self)   -> bool { self.citizen_state().active.get() }
    fn is_selected(&self) -> bool { self.citizen_state().selected.get() }
}
```

`CitizenState` is entirely reactive — every field is a
`Dynamic<T>` from `egui_mobius_reactive`, so other panels and backend
threads observe changes without polling:

```rust
pub struct CitizenState {
    pub active:   Dynamic<bool>,      // the active one in its group
    pub clicked:  Dynamic<bool>,      // true the frame it was clicked
    pub selected: Dynamic<bool>,      // persistent selection toggle
    pub moved:    Dynamic<bool>,      // relocated in the dock tree
    pub location: Dynamic<[f32; 2]>,  // current dock position, if tracked
    pub visible:  Dynamic<bool>,      // currently visible
}
```

Cloning a `CitizenState` shares the underlying storage — the dispatcher
and the panel hold clones that point at the same `Dynamic<T>`.

The `Dispatcher` is the hub. Its core operation, `activate(id)`, is an
encoded set/reset: the named citizen's `active` goes true, every other
goes false, and `Activated` / `Deactivated` messages are queued for
backend routing. That **one-hot activation** is a focus arbiter — at
most one citizen is active at a time, which removes a whole class of
contention between panels.

```text
Tab click
  → dispatcher.activate("alpha")
    → alpha.active = true            (reactive, immediate)
    → beta.active  = false
    → queue ← [Activated{alpha}, Deactivated{beta}]
  → dispatcher.drain_messages()      → route to backend threads
```

The dispatcher keys citizens by `CitizenId`. That detail becomes
load-bearing for breakaway — see the invariant below.

## Why this is not ELM

The Citizen pattern keeps Elm's *observability* without Elm's
*rigidity*. Elm forces every change through one `update` over one
central immutable model. The Citizen pattern instead:

- **distributes the model** into per-panel `CitizenState`, each
  observable on its own, and
- **separates routing from state** — the Dispatcher routes messages;
  the `Dynamic<T>` fields carry observed state.

It is closer to *signals plus commands* than to centralized MVU. The
one-hot active flag in the Dispatcher is the focus arbiter a pure
reactive system usually lacks — a genuine addition, not Elm-minus-rules.

A widget or "atom" inside a Citizen can send messages to a backend
thread through the Dispatcher, but only when the author explicitly wires
it to — message-passing is opt-in, not ambient.

## The goal: breakaway panels

The motivating feature: a panel that lives inside the app can be dragged
**out** of it and become its own top-level window — and dragged back.
egui supports multiple viewports; the historical blocker for this in
egui_dock was that drag-and-drop of tabs *between* windows was not
supported, so detaching a tab by drag had nowhere to go.

The Citizen pattern reframes the problem so that drag-between-windows is
no longer the gating requirement.

### The reframe

A naive detach tries to move a widget tree from one OS window into
another — the hard problem egui_dock stalled on. With self-contained
reactive `CitizenState`, you do not move the tree. You **re-host the
same Citizen in a second viewport**, and the Dispatcher keeps routing to
it by `CitizenId`. The dock tree simply loses a node; the Citizen and
its state are untouched.

Detach becomes "render this citizen over there," not "move UI across
windows." That is a fundamentally easier problem — and it is only easy
*because* the state was never trapped in the dock model in the first
place.

`CitizenState` already carries the fields this needs: `location` for
where it sits, `moved` for relocation, `visible` for whether it is
currently shown.

### The one invariant that matters

The Dispatcher routes by `CitizenId`, and registers citizens in a
`HashMap<CitizenId, CitizenState>`. So:

> **`CitizenId` must stay stable across a detach / re-attach.**

If detaching ever regenerates the id, in-flight messages orphan and the
re-hosted panel loses its backend wiring. Nail id stability first;
everything else in breakaway is plumbing.

## Build approach: thin layer, not a fork

Three options, worst to best:

1. **Reimplement egui_dock inside egui_mobius.** Rejected. egui_dock has
   already solved splits, tab bars, drag-rearrange, serialization. That
   is months of work re-solving solved problems, with ongoing
   maintenance drift against upstream.
2. **Fork egui_dock.** Only if its public API genuinely cannot express
   "this tab wants to leave the tree." A fork is a maintenance tax
   forever.
3. **Thin layer over unmodified egui_dock.** Preferred. The Citizen
   integration is already additive — citizens hook in through the tab
   viewer without touching egui_dock internals. Breakaway should follow
   the same shape: a wrapper that detects a detach gesture, removes the
   node, and spawns a viewport that re-hosts the citizen by id.

If the thin layer hits a wall in egui_dock's API, prefer an **upstream
contribution** over a fork — the maintainer has expressed interest in
revisiting multi-viewport, so the detach hook may be welcome there.

### Sketch

- A wrapper tracks, per citizen, whether it is docked or detached.
- On detach: remove the node from the `DockState`, set the citizen's
  `visible` / `location`, open an `egui` viewport whose UI calls the
  same citizen's render.
- The Dispatcher is unchanged — it keeps routing by `CitizenId`
  regardless of which window the citizen renders in.
- On re-attach: close the viewport, insert the node back into the tree.
- Drag-to-reattach across windows may still need egui-level d&d support;
  a menu / button detach and re-dock works regardless and is the
  pragmatic first cut.

## Proof points

Three citizens already exist, built and used, all hosted by egui_dock:

- **`egui_quill`** — text editor.
- **`egui_lens`** — logger.
- **`egui_grafica`** — node editor / graphics canvas.

`egui_grafica` is the heaviest test of the seam: a scene `Registry`, an
interaction state machine, and a retained wgpu rendering pipeline, all
self-contained behind the Citizen interface. If that detaches into its
own window cleanly, the pattern is proven for arbitrarily complex
panels.

## Open questions

- Drag-to-reattach between windows — wait on egui d&d, or detach via
  menu first?
- One-hot activation *across* windows — is "active" still global, or
  per-window-group?
- Viewport lifecycle on app close — detached windows need to fold back
  into the saved layout.
