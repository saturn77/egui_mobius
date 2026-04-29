# Common pitfalls

Six foot-guns that real `egui_citizen` apps hit. Each is presented as
a concrete broken snippet, an explanation of *why* it fails, and the
fix. None of them produce a panic or compile error — every one is a
silent bug, which is what makes them worth their own chapter.

## 1. Constructing `CitizenState` fresh per frame

**Broken:**

```rust,ignore
match tab.kind {
    TabKind::Drc => DrcPanel::new(CitizenState::default())
                       .show(ui, &mut self),
}
```

**What goes wrong:** `CitizenState::default()` allocates fresh
`Arc<Mutex<...>>` storage that the dispatcher knows nothing about.
The dispatcher's `activate(&drc_id)` writes to *its* table; the panel
reads from *its* freshly-allocated state; the two never agree. The
DRC tab still highlights when clicked (egui_dock handles its own
visual state), but anything reading `drc_state.active.get()` from
elsewhere in the app sees `false` forever. Reactivity is silently
severed.

**Fix:** obtain the `CitizenState` from `Dispatcher::register()`,
store it somewhere durable (the app struct), and clone it into the
per-frame panel:

```rust,ignore
struct App {
    dispatcher: Dispatcher,
    drc_state:  CitizenState,    // registered once, lives on the app
}

impl App {
    fn new(_: &eframe::CreationContext) -> Self {
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

Panels can be stateless; their `CitizenState` cannot. See
[Stored vs stateless panels](patterns/stored-vs-stateless.md) and
[Reactive lifecycle: the trap](concepts/state.md#the-trap-that-bites-everyone).

## 2. Forgetting `drain_messages()`

**Broken:**

```rust,ignore
fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
    DockArea::new(&mut self.tabs).show(ctx, &mut self.tab_viewer);
    // (no dispatcher.drain_messages() anywhere)
}
```

**What goes wrong:** `Dispatcher::activate()` and any explicit
`Dispatcher::send()` calls push into an internal `Vec<CitizenMessage>`
that has no upper bound. If nothing drains it, the vec grows
forever. The app keeps running, the UI keeps rendering, but RSS
climbs every minute the user holds the app open. No panic, no error
log — just a slow leak.

**Fix:** drain once per frame, after `DockArea::show()`:

```rust,ignore
fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
    DockArea::new(&mut self.tabs).show(ctx, &mut self.tab_viewer);

    for msg in self.dispatcher.drain_messages() {
        // process or forward
    }
}
```

If you have nothing to do with the messages yet, drain into an
ignored binding (`let _ = self.dispatcher.drain_messages();`) so the
queue still empties. Don't leave the dispatcher's queue
unattended — ever.

## 3. Calling `activate()` every frame unconditionally

**Broken:**

```rust,ignore
fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
    // The author wanted "this tab is rendering, mark it active":
    self.dispatcher.activate(&tab.citizen_id());
    tab.show(ui, self.app);
}
```

**What goes wrong:** `ui()` runs every frame for every visible tab.
Each call to `activate()` emits an `Activated` message — even when
the citizen was already active in the previous frame. The queue
fills with redundant `Activated` events at the frame rate (60+/sec).
Backend consumers see "fetch was just activated" 60 times a second
and either dedupe defensively or kick off 60 fetches.

When two tabs are visible side-by-side, it's worse: each frame
deactivates the other, so consumers see a flood of
`Activated`/`Deactivated` pairs in alternation.

**Fix:** call `activate()` from `TabViewer::on_tab_button` gated on
`response.clicked()`, never from `ui()`:

```rust,ignore
fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
    if response.clicked() {
        self.dispatcher.activate(&tab.citizen_id());
    }
}

fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
    // ui() only renders. It does not write lifecycle state.
    tab.show(ui, self.app);
}
```

This is the single most common foot-gun and the reason
[the egui_dock background chapter](background/egui_dock.md) draws the
distinction between `ui()` (render) and `on_tab_button` (event) so
sharply. `ui()` is for rendering; `on_tab_button` is for state
transitions. Keep them separate.

## 4. Mixing panel-local state into `CitizenState`

**Broken:**

```rust,ignore
// Trying to model "this panel's slider value" through CitizenState
// (which has fixed library-defined fields):
self.citizen_state.active.set(true);  // hijacking `active` as
                                      // "is showing project X"
```

…or, equivalently:

```rust,ignore
struct LoggerPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    log_buffer: Dynamic<Vec<LogEntry>>,  // overkill; nothing else
                                         // outside the panel reads it
}
```

**What goes wrong:** `CitizenState` has a fixed shape — six
`Dynamic<T>` fields with library-defined semantics (active, clicked,
selected, moved, location, visible). Reusing those fields to mean
something domain-specific (e.g. `active` = "showing project X")
breaks the dispatcher's invariants the moment you call
`activate()`, which clobbers your overload.

The variant is treating *every* panel-local field as reactive
"because reactivity feels good." Reactivity is not free — every
`Dynamic<T>` is an `Arc` plus a lock plus a notifier list. If only
the panel itself reads its log buffer, a plain `Vec<LogEntry>` is
the right type.

**Fix:** use a separate `PanelState` struct alongside `CitizenState`
on the panel struct, and use plain (non-reactive) types unless a
second reader actually exists:

```rust,ignore
struct LoggerPanelState {
    log_buffer: Vec<LogEntry>,
    filter_text: String,
    follow_tail: bool,
}

struct LoggerPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,    // library lifecycle, fixed
    panel_state: LoggerPanelState,  // panel-author's fields
}
```

See [Where does state live?](patterns/state-shape.md) for the full
three-bucket model.

## 5. Expecting `visible` to track egui_dock's open/closed state

**Broken:**

```rust,ignore
// Assumes `visible` updates automatically when egui_dock opens or
// closes the tab — it doesn't.
if self.plot_state.visible.get() {
    self.start_streaming_data();
}
```

**What goes wrong:** `egui_citizen` and `egui_dock` are independent
crates. `egui_dock` does not know `egui_citizen` exists, so it never
calls `state.visible.set()` on its own. The `visible` field starts
at `false`, stays at `false`, and your "is the panel showing?" check
returns `false` even when the user is staring at the panel.

**Fix:** drive `visible` yourself, either by setting it directly
when you detect a tab open/close in `TabViewer`, or — preferred —
by routing through the [`VisibilityChanged`](concepts/messages.md)
message:

```rust,ignore
// Detect close in TabViewer:
fn on_close(&mut self, tab: &mut Tab) -> bool {
    self.dispatcher.send(CitizenMessage::VisibilityChanged {
        id: tab.citizen_id(),
        visible: false,
    });
    true
}

// In the drain loop, sync the reactive flag:
for msg in self.dispatcher.drain_messages() {
    if let CitizenMessage::VisibilityChanged { id, visible } = &msg {
        if let Some(state) = self.dispatcher.get(id) {
            state.visible.set(*visible);
        }
    }
}
```

`egui_citizen` provides the *vocabulary* (a `Dynamic<bool>` field, a
message variant). The plumbing from egui_dock's tab-close into that
vocabulary is your code's responsibility, by design — it's the
boundary that keeps `egui_citizen` independent of the dock crate.

## 6. Two dispatchers in one app

**Broken:**

```rust,ignore
struct App {
    plot_dispatcher:     Dispatcher,
    settings_dispatcher: Dispatcher,
    /* ... */
}
```

**What goes wrong:** the one-hot activation invariant is
**per-dispatcher**. `plot_dispatcher.activate(&plot_id)` deactivates
every other citizen registered with `plot_dispatcher` — but it
cannot deactivate a citizen registered with `settings_dispatcher`,
because the two dispatchers maintain entirely separate
`HashMap<CitizenId, CitizenState>` tables. Two panels — one
registered to each dispatcher — can both be "active" simultaneously,
which the rest of the codebase does not expect.

**Fix:** one `Dispatcher` per app. Always.

```rust,ignore
struct App {
    dispatcher: Dispatcher,    // exactly one
    /* ... */
}
```

If you find yourself wanting a second dispatcher because "these
panels are unrelated to those panels," the right answer is still one
dispatcher with all panels registered. Citizen ids are namespaced
strings — use `"editor.plot"`, `"sidebar.settings"`, etc., to
disambiguate. The dispatcher does not care about logical grouping;
it only enforces the one-hot invariant across everything it knows
about.

## Summary

Five of the six foot-guns above are silent — no panic, no compile
error, just behavior that drifts from what the author expected. The
defenses are vocabulary-level: hold the `CitizenState` somewhere
durable, drain the queue every frame, never write lifecycle state in
`ui()`, keep panel-local data out of `CitizenState`, drive `visible`
yourself, and never run two dispatchers in one app. Once these
become habits, the rest of `egui_citizen` works out of the box.
