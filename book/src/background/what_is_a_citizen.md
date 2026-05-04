# What is a citizen?

Before any of the trait, dispatcher, or reactive-state machinery,
get the picture from the UI side first.

> **A citizen is a panel.** A docked, movable, resizable region of
> the application window with a stable identity and a known set of
> widgets inside it.

That's the user-facing definition. Everything else — the `Citizen`
trait, the `Dispatcher`, the reactive `CitizenState` — is the
plumbing that makes the behavior of those panels predictable across
an application. The plumbing matters, but the panel is what the user
actually sees and interacts with.

## The general characteristics

A citizen panel has all of the following:

- **Identity.** Each panel has a stable name (a `CitizenId` —
  `"plot"`, `"settings"`, `"logger"`, `"gerber_view"`). Two panels
  cannot share an identity within the same app. The identity
  outlives any individual frame and survives layout changes.

- **Dockable.** The panel slots into the application's dock layout
  via `egui_dock` (or a sibling dock library). It can sit alongside
  other citizen panels in a tabbed group, in a split, or as a
  free-floating window.

- **Movable.** The user can drag the panel by its tab bar to a
  different dock position. Citizen identity is preserved through
  the move — the panel knows it's still the same panel after
  landing in a new spot.

- **Resizable.** Dock split handles let the user reapportion space
  between citizens. The panel adapts; it does not lose state when
  its size changes.

- **Atomic content.** A panel contains **atoms** — the widgets
  inside it: a slider, a button, a checkbox, a text field, a
  scrollable list, a plot. Atoms are the panel's interactive
  surface. A panel without atoms is a static label; the citizen
  pattern shines when atoms drive shared state that other citizens
  observe.

- **Lifecycle awareness.** At any moment exactly one citizen is
  the *active* one in its group. Activation flips when the user
  clicks the panel's tab. The pattern guarantees this is exclusive
  — when "alpha" activates, "beta" deactivates atomically. Other
  panels can react to this without polling.

- **Reactive state.** A small bundle of `Dynamic<T>` cells —
  active, clicked, selected, visible, location, moved — published
  by every citizen and readable from anywhere in the application.
  Other panels and backend threads observe these without holding
  references to the panel itself.

## What this buys you that `egui_dock` alone doesn't

`egui_dock` is a layout library. It tells you which panel is
*visible* on screen and which sits in which split. It does not
tell you which panel is currently the **active citizen of
interest** — i.e., which panel the user clicked last, which one
should be receiving keyboard focus, which one a backend thread
should be feeding fresh data to.

The citizen pattern fills that gap by giving every panel its
**own reactive state** (`CitizenState`) and routing tab clicks
through a central `Dispatcher`. The dispatcher's `activate(...)`
call is an atomic set/reset: when "alpha" becomes active, every
other citizen's `active` cell flips to `false` in the same
operation, and lifecycle messages (`Activated { id: alpha }`,
`Deactivated { id: beta }`) drop into the dispatcher's queue.

This means:

- A **backend thread** can poll the dispatcher (or observe each
  citizen's `active` `Dynamic<bool>`) and discern *which citizen
  is currently of interest* without holding a reference to any
  panel. Background work — fetching data, running computations,
  reading hardware — knows where to direct its results.
- A **sibling panel** can react to another panel becoming active
  without any per-frame polling: the reactive cell delivers the
  change, the rendering panel re-reads on its next frame.
- The pattern guarantees **one-hot activation** — exactly one
  citizen active at a time per group — atomically. Two panels
  can never both think they're active because of a frame-order
  race. `egui_dock` makes no such guarantee; you'd have to wire
  it manually.

This separation — dock library handles geometry, citizen
dispatcher handles *interest* — is the load-bearing distinction.
Without it, every app reinvents some ad-hoc "which panel did the
user mean?" logic. With it, that's framework infrastructure you
inherit for free.

## Citizens as plug-ins

The other consequence of the citizen contract is that **citizens
become plug-ins**. Once a panel implements the `Citizen` trait,
exposes its reactive state, and integrates with the dispatcher,
it doesn't need to know anything about the host app to drop in.
The host app, conversely, just needs to:

1. Add the citizen's crate as a dependency.
2. Carry its `Dynamic<T>` state field on the shared state struct.
3. Add a `TabKind` variant for it.
4. Render it from the `TabViewer`.

That's the whole integration. No glue code, no event-bus wiring,
no manual subscription setup. The citizen pulls its weight as a
self-contained unit.

`egui_lens` (the reactive event logger) and `egui_quill` (the
syntax-highlighted editor) are the canonical examples of this in
action. Both ship as their own workspace crates with stable
public APIs (`ReactiveEventLogger` + `ReactiveEventLoggerState`;
`ReactiveEditor` + `ReactiveEditorState`). The host app's
integration is small enough to fit on a sticky note:

```rust,ignore
// One field on shared state.
pub log: Dynamic<ReactiveEventLoggerState>,

// One render call per frame.
let logger = ReactiveEventLogger::new(&state.log);
logger.show(ui);
```

Same pattern for quill, same pattern for the canonical citizen
panels coming next (Project / Settings / Terminal / Data Table —
issue #33). Same pattern for any third-party citizen someone
publishes on crates.io: `cargo add egui_their_panel`, declare
the state field, render in a tab, done.

This is what makes the framework genuinely composable rather than
just architecturally tidy. Real apps grow by accumulating
citizens, not by extending their core.

## What it is not

A citizen is not:

- A modal dialog or a popover. Those have transient lifetimes and
  no stable identity.
- A widget. Widgets are atoms; they live *inside* citizens.
- A non-docked sub-region of a single window. The dockability is
  intrinsic; without it, a panel is just a layout container.
- A backend thread. Background work runs separately and
  communicates with citizens through reactive state and message
  channels.

## A picture

```text
┌───────────────────────────────────────────────────────────────┐
│ App window                                                     │
│ ┌──────────────┬──────────────────────┬──────────────────────┐ │
│ │ ▶ Settings ✕ │ ▶ Plot ✕             │ ▶ Logger ✕           │ │
│ ├──────────────┼──────────────────────┼──────────────────────┤ │
│ │              │                      │                      │ │
│ │  citizen     │     citizen          │      citizen         │ │
│ │  "settings"  │     "plot"           │      "logger"        │ │
│ │              │                      │                      │ │
│ │  atoms:      │     atoms:           │      atoms:          │ │
│ │  - slider    │     - plot widget    │      - filter btn    │ │
│ │  - combobox  │     - link checkbox  │      - clear btn     │ │
│ │  - generate  │                      │      - save btn      │ │
│ │    button    │                      │      - column toggles│ │
│ │              │                      │      - scroll list   │ │
│ └──────────────┴──────────────────────┴──────────────────────┘ │
└───────────────────────────────────────────────────────────────┘
```

Three citizens. Each is dockable / movable / resizable. Each holds
atoms that are the user's actual interaction surface. Reactive
state flows through the framework's plumbing so the plot panel can
react to a settings-panel atom, and the logger can show a
backend-thread message, without any of them needing direct
references to the others.

## Where the rest of the book goes from here

- The [Citizen trait chapter](../concepts/citizen.md) is the code
  shape: what `impl Citizen for MyPanel` looks like, the lifecycle
  hooks, the `CitizenId` and `CitizenState` types.
- The [Dispatcher chapter](../concepts/dispatcher.md) is the
  coordinator: how activation propagates and how messages drain.
- The [`Dynamic<T>` background chapter](dynamic_type.md) is the
  reactive primitive every `CitizenState` field rests on.
- The [tutorial](../tutorial/writing-a-citizen-app.md) is the
  worked example — three citizens (Plot / Settings / Logger),
  built end to end.

---

*Chapter last revised: 2026-05-04 — egui_mobius v0.4.0.*
