# Stored vs stateless panels

Two lawful ways to use a citizen, both correct, and the choice depends
on what state the panel itself holds.

- **Stored** — the panel is a field on the app struct, constructed
  once in `App::new()`, rendered via `self.panel.show(ui, ...)` each
  frame. Panel-local state survives between frames because the panel
  struct does.
- **Stateless per-frame** — the panel is constructed fresh inside the
  `TabKind` dispatch arm of your `TabViewer`, used once, dropped at
  the end of `ui()`. Anything panel-local is wiped between frames;
  the panel relies on app-shared services for everything it needs
  to render.

Both forms work. Pick the one that matches the state the panel owns.

## Stored panels

```rust,ignore
struct App {
    dispatcher: Dispatcher,
    logger: LoggerPanel,        // stored: lives across frames
    bom:    BomPanel,           // stored
    /* ... */
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        let mut dispatcher = Dispatcher::new();

        let logger = LoggerPanel::new(
            dispatcher.register(CitizenId::new("logger")),
        );
        let bom = BomPanel::new(
            dispatcher.register(CitizenId::new("bom")),
        );

        Self { dispatcher, logger, bom }
    }
}
```

In your `TabViewer::ui`, the panel field gets passed by mutable
reference:

```rust,ignore
match tab.kind {
    TabKind::Logger => self.logger.show(ui, &self.services),
    TabKind::Bom    => self.bom.show(ui, &self.services),
    /* ... */
}
```

**Use stored panels for any panel that owns non-trivial local state.**
Concretely:

- **Accumulating buffers** — log entries, terminal scrollback,
  command history.
- **Caches** — image / texture caches, parsed file caches, computed
  layout caches.
- **Per-panel UI state that must persist** — scroll position (when
  egui's own memory doesn't already handle it), filter text, modal
  open-state.

Anything that should *not* vanish between frames belongs in a stored
panel's `PanelState` (see
[Where does state live?](state-shape.md)).

## Stateless per-frame panels

```rust,ignore
match tab.kind {
    TabKind::Drc      => DrcPanel::new(self.drc_state.clone())
                            .show(ui, &mut self),
    TabKind::Settings => SettingsPanel::new(self.settings_state.clone())
                            .show(ui, &mut self),
    /* ... */
}
```

The panel struct is constructed every frame, used once, dropped. It
holds no panel-local fields beyond its `CitizenId` and `CitizenState`
— everything else it renders comes from `&self` (the app) or
`&mut services`.

**Use stateless panels when the panel is a pure view** over data
that already lives somewhere else:

- **DRC results panel** — DRC results live in shared services
  (computed from the project model). The panel is a view; nothing
  needs to survive between frames.
- **View settings panel** — settings are shared application state.
  The panel reads and writes them through `&mut services`, never
  caching anything locally.
- **Project picker / file picker** — the project list comes from the
  filesystem each frame, or from a service that caches it; the panel
  itself doesn't.

## The trap that kills reactivity

This is the single most common foot-gun, and it is **silent** — no
panic, no error, no warning. Reactivity quietly stops working.

The stateless form *looks* like it should work with
`CitizenState::default()`:

```rust,ignore
// WRONG — fresh storage, disconnected from the dispatcher
match tab.kind {
    TabKind::Drc => DrcPanel::new(CitizenState::default())  // ← !!!
                        .show(ui, &mut self),
}
```

The panel constructs, renders, and drops cleanly. The dispatcher's
`activate(&drc_id)` runs without complaint. The DRC tab even
highlights when clicked, because `egui_dock` handles its own visual
state.

But: any code that reads `drc_state.active.get()` from *outside* the
DRC panel reads from a `CitizenState` that the dispatcher knows
nothing about — because the panel constructed its own with
`::default()`. Subscribers across the app see the value never change,
even though the dispatcher's internal table says the DRC citizen is
active. The dispatcher's storage and the panel's storage are two
completely different `Arc`s.

The fix: always obtain the `CitizenState` from
`dispatcher.register()`, store it somewhere durable, and clone it
into the per-frame panel.

```rust,ignore
struct App {
    dispatcher: Dispatcher,
    drc_state:  CitizenState,    // stored on the app even though
                                 // the panel itself is stateless
    /* ... */
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        let mut dispatcher = Dispatcher::new();
        let drc_state = dispatcher.register(CitizenId::new("drc"));
        Self { dispatcher, drc_state }
    }
}

// In TabViewer:
match tab.kind {
    TabKind::Drc => DrcPanel::new(self.drc_state.clone())
                        .show(ui, &mut self),
}
```

The `CitizenState` lives on the app struct (so it survives across
frames and the dispatcher and panel agree on storage); the *panel
struct itself* is still constructed fresh. Reactivity works because
the `Arc` clones share underlying storage (see
[Reactive lifecycle: clones share storage](../concepts/state.md#clones-share-storage--this-is-the-whole-game)).

**The shorthand: panels can be stateless; their `CitizenState`
cannot.**

## Mixing the two in one app

Real apps mix freely. CopperForge — a non-trivial example — keeps
**stored** panels for those that own buffers or caches:

| Stored panel       | Reason                                       |
|--------------------|----------------------------------------------|
| `logger`           | log buffer accumulates over the session      |
| `bom`              | parsed / cached BOM rows                     |
| `terminal`         | shell scrollback                             |
| `shell`            | command history                              |
| `gerber_view_3d`   | geometry cache + camera state                |

…and **stateless** for panels that are pure views over shared data:

| Stateless panel       | Reason                                            |
|-----------------------|---------------------------------------------------|
| `DrcPanel`            | DRC results live in shared services               |
| `ViewSettingsPanel`   | settings live in shared services                  |
| `SettingsPanel`       | configuration lives in shared services            |
| `ProjectsPanel`       | project list comes from filesystem / services     |

The dispatcher is unaware of the difference. From its perspective,
every citizen is just `(CitizenId, CitizenState)`, regardless of
whether the surrounding panel struct is stored or constructed
per-frame. The split is purely a question of where panel-local state
lives.

## Decision rule

Ask one question:

> **Does this panel own state that must survive between frames?**

| Answer | Use         |
|--------|-------------|
| Yes    | Stored      |
| No     | Stateless   |

State that "must survive" includes: buffers, caches, per-panel UI
state that egui itself doesn't remember, accumulated history. State
that does *not* count: anything you can re-derive from app-shared
services or read from the panel's `CitizenState` flags.

If you're unsure, default to **stored**. The cost of a panel with
no local state being stored is one extra struct field; the cost of
a stateless panel that secretly needed local state is hours of
debugging why something doesn't update across frames.

## Summary

- Stored panels are app fields constructed once. Use when the panel
  owns local state that must persist across frames.
- Stateless panels are constructed per-frame in the tab-dispatch arm.
  Use when the panel is a pure view over shared services.
- **The `CitizenState` always comes from
  `dispatcher.register()` and lives somewhere durable** — on the
  panel struct for stored, on the app struct for stateless.
  Constructing it with `::default()` silently severs reactivity.
- When in doubt, choose stored.
