# Where does state live?

State is a struct. Where it lives — and which struct it lives in —
is the difference between code that scales and code that fights
itself. A real `egui_citizen` app has three of these structs, sitting
in three different places.

## The three structs

### 1. `CitizenState` — lifecycle facts only

The library's [`CitizenState`](../concepts/state.md) holds *lifecycle*
data: is this panel active, was it clicked this frame, is it visible,
has it moved. Six fields, fixed shape, all `Dynamic<T>`. Reactive by
design — other panels and threads can read these values directly.

What goes here: questions the dock or other panels ask about *this
panel's status*.

What does **not** go here: business data. `CitizenState` is a shared
library type with a fixed contract. Don't try to extend it with
domain-specific fields.

### 2. `PanelState` — your panel-local struct

Whatever the panel needs to do its own job that nobody else reads or
writes. By convention, give it its own struct named `FooPanelState` (or
just `PanelState` if scoped inside a panel module). These fields don't
need to be reactive — only the panel itself touches them.

```rust,ignore
struct LoggerPanelState {
    log_buffer: Vec<LogEntry>,
    filter_text: String,
    follow_tail: bool,
}

struct LoggerPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,   // bucket 1: library-defined lifecycle
    panel_state: LoggerPanelState, // bucket 2: panel-local data
}
```

What goes in `PanelState`: UI scratch state, caches, buffers,
modal-open flags, filter text, scroll positions, accumulated log
entries.

Why a struct instead of loose fields on the panel? Three reasons:

1. **It names the bucket.** A reader scanning `LoggerPanel` sees three
   things — id, citizen state, panel state — instead of a flat list
   that mixes concerns.
2. **It mirrors `CitizenState`.** Both are "state for one panel,"
   one library-defined and one app-defined. The parallel makes the
   design rule visible.
3. **It survives refactors.** When the panel grows, panel-local fields
   stay clustered. When you eventually need to persist or snapshot
   panel state, it's already a single value.

For tiny panels with one or two fields, inlining on the panel struct
is fine — just promote to a named struct the moment a third field
appears.

### 3. App-shared state — data many panels touch

Anything two or more panels need to read or mutate. Project config, a
database handle, the layer store in a CAD app, a `SharedServices`
struct passed by reference into every panel's `show()`.

```rust,ignore
struct App {
    services: Arc<SharedServices>,  // shared
    dispatcher: Dispatcher,          // shared
    logger: LoggerPanel,             // owns its panel-local state
    bom: BomPanel,
}

fn show(&mut self, ui: &mut egui::Ui) {
    self.logger.show(ui, &self.services);
    self.bom.show(ui, &self.services);
}
```

Whether the shared bits are themselves reactive (`Dynamic<T>` inside
`SharedServices`) or guarded by `Arc<Mutex<...>>` is a separate design
choice. The point is: shared data lives at the app level, not stuffed
inside `CitizenState` and not duplicated across panel structs.

## The rule of thumb

Ask in this order:

| Question                          | Bucket               |
|-----------------------------------|----------------------|
| Is this a lifecycle fact?         | `CitizenState`       |
| Do two or more panels need it?    | App-shared           |
| Otherwise                         | `PanelState`         |

If a piece of data fits the lifecycle list — active, clicked, selected,
moved, location, visible — it belongs in `CitizenState`. If not, ask
whether anything outside the panel reads or mutates it. If yes,
app-shared. Otherwise, `PanelState`.

## Anti-patterns

**Adding business fields to `CitizenState`.** `CitizenState` has a
fixed shape from the library. Wrap it inside your own panel struct
alongside your fields — don't try to extend it.

```rust,ignore
// WRONG — won't compile, and shouldn't
struct CitizenState {
    active: Dynamic<bool>,
    // ...
    bom_rows: Vec<BomRow>, // no
}

// RIGHT
struct BomPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    bom_rows: Vec<BomRow>,
}
```

**Duplicating shared data across panels.** If three panels need to
read the project config, don't give each one its own copy and try to
synchronize. Put it in `SharedServices` (or whatever your app calls
its shared bag) and pass `&services` into each panel's `show()`.

**Putting `PanelState` fields in `Dynamic<T>` "in case someone needs
them later."** Reactivity is not free — every `Dynamic<T>` is an `Arc`
plus a lock. If only the panel reads its own filter text, a plain
`String` field is fine. Promote to `Dynamic<T>` the day a second
reader actually appears.

## Worked example

A CAD app with three panels: `BomPanel` (lists components), `DrcPanel`
(design rule check results), `ViewSettings` (camera and grid options).

| Data                              | Bucket          | Why                                        |
|-----------------------------------|-----------------|--------------------------------------------|
| `BomPanel.citizen_state.active`   | `CitizenState`  | Dock asks which panel is active.           |
| `BomPanel.panel_state.search_text`| `PanelState`    | Only `BomPanel` reads it.                  |
| `BomPanel.panel_state.cached_rows`| `PanelState`    | Cached view of project data, panel's own.  |
| `project.components[]`            | App-shared      | DRC reads it; BOM mutates it.              |
| `view.camera`                     | App-shared      | Viewport reads, settings writes.           |
| `ViewSettings.panel_state.is_dirty`| `PanelState`   | Only `ViewSettings` tracks pending edits.  |

The split scales. Adding a fourth panel that reads
`project.components[]` is a one-line change — accept `&SharedServices`
in its constructor — not a refactor.

## What about activation-driven business state?

A common variant: "when panel A activates, panel B should switch to a
particular view of the shared data." That data still lives in
app-shared state — the *trigger* for switching the view is panel A's
`citizen_state.active`, which panel B reads reactively. Lifecycle
drives the transition; the data being viewed never moves into
`CitizenState`.

## Summary

Three structs, in priority order: `CitizenState` for lifecycle facts,
app-shared services for cross-panel data, `PanelState` for everything
else. Don't extend `CitizenState`. Don't duplicate shared data. Don't
reach for `Dynamic<T>` until a second reader exists.
