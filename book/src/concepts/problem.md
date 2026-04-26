# The problem

The [introduction](../introduction.md) sketched the per-frame race in
broad strokes. This chapter pins down *exactly* why the race happens
in an `egui_dock` app, and exactly which API mechanic the rest of the
book is built around.

## Every visible tab's `ui()` runs every frame

`egui_dock` is an immediate-mode dock. There is no "active tab"
notion baked into its rendering: every tab that is currently visible
in the dock layout — meaning its node is rendered, even if other
nodes are also visible alongside it — has its `ui()` callback fire
**every single frame**.

This is by design. egui as a whole is immediate-mode: the entire UI
is reconstructed each frame from scratch. `egui_dock` extends that
model to multiple panels in a dock layout. Visibility, not a
focus/active flag, drives whether `ui()` fires.

The implication is small but load-bearing: **`ui()` is a render
callback, not an event hook**. Anything you do inside it happens
once per frame per visible tab, not once per user action.

## Writing an `egui_citizen` app means implementing `TabViewer`

The bridge between `egui_dock` and your application is a single
trait, `egui_dock::TabViewer`, that you implement. Its skeleton looks
like this:

```rust,ignore
struct MyTabViewer<'a> {
    app: &'a mut App,                   // your app's shared state
    dispatcher: &'a mut Dispatcher,     // egui_citizen's dispatcher
}

impl egui_dock::TabViewer for MyTabViewer<'_> {
    type Tab = MyTab;                   // your tab type

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title.clone().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        // RENDERING. Runs every frame, for every visible tab.
        tab.show(ui, self.app);
    }

    fn on_tab_button(
        &mut self,
        tab: &mut Self::Tab,
        response: &egui::Response,
    ) {
        // STATE TRANSITIONS. Fires once when the tab is clicked.
        if response.clicked() {
            self.dispatcher.activate(&tab.citizen_id());
        }
    }
}
```

That impl is not optional, it is the *integration shape* of any
`egui_dock` + `egui_citizen` app. Two of its methods are the topic of
this chapter: `ui` and `on_tab_button`. Their roles are entirely
distinct, and conflating them is the root of the per-frame race.

## The wrong-hook trap: state transitions in `ui()`

Suppose a naive author wants to track which tab is "active." They
write this inside `ui()`, reasoning that *this tab is rendering, so
mark it active*:

```rust,ignore
fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
    self.app.active_tab = tab.kind;   // ← every visible tab does this
    tab.show(ui, self.app);
}
```

What actually happens, given `egui_dock`'s rendering model:

- Frame N renders. Tab Plot is visible, Tab Logger is visible.
- `ui()` fires for Plot — `app.active_tab = Plot`.
- `ui()` fires for Logger — `app.active_tab = Logger`.
- Last write wins. `active_tab` is `Logger`.
- Frame N+1 renders. The cycle repeats. `active_tab` flickers
  between `Plot` and `Logger` every frame depending on render order.

The race is inherent. It does not go away with locking, refactoring,
or moving the assignment into a method. As long as state-transition
logic lives in a callback that fires per-frame-per-visible-tab, the
last visible tab to render wins, every frame, regardless of which
tab the user actually interacted with.

This is the foot-gun. Most authors hit it, work around it with ad-hoc
"who clicked last?" hacks (a `click_time` epoch, an `is_focused`
field that flips itself every frame, etc.), and the workarounds
don't scale.

## The right hook: `on_tab_button` with `response.clicked()`

`egui_dock` does provide a one-shot click hook. It is
`TabViewer::on_tab_button`, and `response.clicked()` is the gate that
distinguishes the click frame from the rendering frames around it:

```rust,ignore
fn on_tab_button(
    &mut self,
    tab: &mut Self::Tab,
    response: &egui::Response,
) {
    if response.clicked() {
        self.dispatcher.activate(&tab.citizen_id());
    }
}
```

This callback runs once per tab button per frame, and the `clicked()`
predicate fires exactly once per actual user click. State transitions
made inside this guard are *single-shot events*, not per-frame
overwrites. The race goes away because the assignment only happens
on the frame the user actually clicked.

## The discoverability problem

Here is the catch: `on_tab_button` is named like a render or styling
callback. It sounds like *"called while drawing the tab button"* —
which it is, but only secondarily. Its primary job, in practice, is
to be the place where `clicked()` is true. The name does not
telegraph that role.

Compared to `ui()` — which sounds exactly like a render callback,
because that is what it is — `on_tab_button` reads as a sibling
"render the button" hook. The natural mental model is "do styling
work in `on_tab_button`, do main work in `ui()`," and that mental
model puts state-transition code in the wrong place.

Names like `on_panel_selected` or `on_focus_changed` would
immediately telegraph "this is a state-transition event hook, not a
render callback." But the API uses `on_tab_button`, and the
distinction it draws between event-time and render-time logic is
precisely the distinction `egui_citizen` exists to enforce.

## `egui_citizen`'s answer

`egui_citizen` makes the event-time / render-time distinction
**concrete and unmissable**:

- The dispatcher exposes one canonical state-transition primitive:
  [`Dispatcher::activate(&id)`](dispatcher.md#activateid).
- That primitive is **only** ever called from `on_tab_button` (or
  equivalent user-driven event hooks). It is never called from
  `ui()`.
- `ui()` reads — `tab.show(ui, ...)`, `self.is_active()`,
  `self.state.active.get()` — but it never writes lifecycle state.
- The dispatcher's queue means the consequences of an `activate()`
  call (the `Activated` / `Deactivated` messages, the reactive flag
  flips) propagate at well-defined boundaries: in the frame's drain
  pass, not partway through a render.

The integration shape becomes:

| Callback          | Role                            | Allowed to do            |
|-------------------|---------------------------------|--------------------------|
| `ui()`            | Render the panel                | Read state. **Never write lifecycle state.** |
| `on_tab_button`   | Detect tab clicks               | Call `dispatcher.activate(&id)` on click. |
| Drain loop        | Process state-change messages   | Mutate app-shared state, forward to backend. |

That separation — events in `on_tab_button`, rendering in `ui()`,
consequences drained once per frame — is what makes a multi-panel
`egui_dock` app stop fighting itself. The rest of this book is the
mechanics of how that works: identities, reactive state, the
dispatcher, the message queue, the coupling paths.

## Summary

- `ui()` runs every frame for every visible tab. It is a render
  callback.
- `on_tab_button` with `response.clicked()` is the one-shot click
  hook. It is the right place for state transitions.
- The name `on_tab_button` does not telegraph that role, which is
  why most authors initially put state-transition code in `ui()` and
  hit the per-frame race. Discoverability is the foot-gun.
- `egui_citizen` enforces the distinction by making
  `Dispatcher::activate()` the canonical state-transition primitive
  and routing it exclusively through `on_tab_button`. `ui()` only
  reads.
