# Reference

A single-page summary of the public API. For full signatures, doc
comments, and version-tracked details, see
[docs.rs/egui_citizen](https://docs.rs/egui_citizen).

## `Dispatcher`

```rust,ignore
use egui_citizen::Dispatcher;
```

| Call                                          | What it does                                             |
|-----------------------------------------------|----------------------------------------------------------|
| `Dispatcher::new()`                           | Empty dispatcher.                                        |
| `register(id) -> CitizenState`                | Register a citizen; return a `CitizenState` handle.      |
| `get(&id) -> Option<&CitizenState>`           | Look up a registered citizen's state.                    |
| `activate(&id)`                               | One-hot: this one on, all others off. Emits messages.    |
| `send(message)`                               | Push a `CitizenMessage` onto the queue without activating.|
| `drain_messages() -> Vec<CitizenMessage>`     | Take all pending messages. Call once per frame.          |
| `len()` / `is_empty()`                        | Citizen count / emptiness.                               |

See [the dispatcher chapter](concepts/dispatcher.md) for the one-hot
invariant and the canonical drain loop.

## `Citizen` trait

```rust,ignore
use egui_citizen::Citizen;
```

| Method                                       | Provided?  | Purpose                                  |
|----------------------------------------------|------------|------------------------------------------|
| `id() -> &CitizenId`                         | required   | Stable identity.                         |
| `citizen_state() -> &CitizenState`           | required   | Read access to lifecycle state.          |
| `citizen_state_mut() -> &mut CitizenState`   | required   | Mutable access to lifecycle state.       |
| `on_activate()`                              | default    | Sets `citizen_state.active = true`.      |
| `on_deactivate()`                            | default    | Sets `citizen_state.active = false`.     |
| `on_click()`                                 | default    | Sets `citizen_state.clicked = true`.     |
| `is_active() -> bool`                        | default    | `citizen_state.active.get()`.            |
| `is_selected() -> bool`                      | default    | `citizen_state.selected.get()`.          |

See [the Citizen trait chapter](concepts/citizen.md) for the
minimum-viable impl and override semantics.

## `CitizenState`

```rust,ignore
use egui_citizen::CitizenState;
```

Six reactive `Dynamic<T>` fields:

| Field      | Type                | Meaning                                       |
|------------|---------------------|-----------------------------------------------|
| `active`   | `Dynamic<bool>`     | This citizen is the active one (one-hot).     |
| `clicked`  | `Dynamic<bool>`     | True for the frame this citizen was clicked.  |
| `selected` | `Dynamic<bool>`     | Persistent selection toggle.                  |
| `moved`    | `Dynamic<bool>`     | Citizen moved to a new dock location.         |
| `location` | `Dynamic<[f32; 2]>` | Last known dock-layout position.              |
| `visible`  | `Dynamic<bool>`     | Citizen is currently visible.                 |

Cloning a `CitizenState` clones the `Arc`s — every clone refers to
the same storage. See [Reactive
lifecycle](concepts/state.md) and [Inside
`Dynamic<T>`](concepts/inside-dynamic.md) for the underlying machinery.

## `CitizenMessage`

```rust,ignore
use egui_citizen::CitizenMessage;
```

| Variant                                     | Emitted by                                                   |
|---------------------------------------------|--------------------------------------------------------------|
| `Activated { id }`                          | `Dispatcher::activate(&id)`                                  |
| `Deactivated { id }`                        | `Dispatcher::activate(&id)` for the previously active citizen|
| `Clicked { id }`                            | App code via `Dispatcher::send`                              |
| `Selected { id, selected: bool }`           | App code via `Dispatcher::send`                              |
| `Moved { id, location: [f32; 2] }`          | App code via `Dispatcher::send`                              |
| `VisibilityChanged { id, visible: bool }`   | App code via `Dispatcher::send`                              |

Derives: `Debug`, `Clone`. See [the messages chapter](concepts/messages.md).

## `CitizenId`

```rust,ignore
pub struct CitizenId(pub String);

impl CitizenId {
    pub fn new(id: impl Into<String>) -> Self { /* ... */ }
}
```

Stable string identifier for a citizen. Define ids as `const`s in
your app to make typos a compile-time error:

```rust,ignore
const PLOT_ID:    &str = "plot";
const LOGGER_ID:  &str = "logger";
```

## Common idioms

### Register and activate

```rust,ignore
let mut dispatcher = Dispatcher::new();
let plot_state = dispatcher.register(CitizenId::new("plot"));
dispatcher.activate(&CitizenId::new("plot"));
```

### Implement `Citizen` on a panel struct

```rust,ignore
struct PlotPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
}

impl Citizen for PlotPanel {
    fn id(&self) -> &CitizenId               { &self.citizen_id }
    fn citizen_state(&self) -> &CitizenState  { &self.citizen_state }
    fn citizen_state_mut(&mut self) -> &mut CitizenState {
        &mut self.citizen_state
    }
}
```

### Wire activation through `egui_dock::TabViewer`

```rust,ignore
fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
    if response.clicked() {
        self.dispatcher.activate(&tab.citizen_id());
    }
}
```

### Drain messages once per frame

```rust,ignore
for msg in self.dispatcher.drain_messages() {
    match msg {
        CitizenMessage::Activated { id } => { /* ... */ }
        CitizenMessage::Deactivated { id } => { /* ... */ }
        _ => {}
    }
}
```

### Wrap in your own `AppMessage`

```rust,ignore
pub enum AppMessage {
    Citizen(CitizenMessage),
    /* domain variants ... */
}

for msg in self.dispatcher.drain_messages() {
    let _ = self.tx_backend.send(AppMessage::Citizen(msg));
}
```

## See also

- [docs.rs/egui_citizen](https://docs.rs/egui_citizen) — full rustdoc.
- [docs.rs/egui_mobius_reactive](https://docs.rs/egui_mobius_reactive) —
  `Dynamic<T>`, `Value<T>`, `Derived<T>`.
- [docs.rs/egui_dock](https://docs.rs/egui_dock) — `DockArea`,
  `TabViewer`.
