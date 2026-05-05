# `egui_lens` — the reactive event logger

> **`egui_lens` is a citizen.** A docked, movable, resizable panel
> with stable identity (`"logger"`) and a known set of atoms inside
> — System Info, Filters, Logger Colors, Save Logs, Clear Logs,
> the column-toggle checkboxes, the scrollable log area itself.
> Like every other citizen panel, it observes shared reactive state
> through `Dynamic<T>` and participates in dispatcher-coordinated
> activation.

If "citizen" doesn't ring a bell yet, read [What is a
citizen?](../background/what_is_a_citizen.md) first — that chapter
walks through the panel-level characteristics every citizen has,
which `egui_lens` is the canonical example of.

## What it does

`egui_lens` is the canonical event logger for the `egui_mobius`
ecosystem. It provides a terminal-style log panel — log levels,
per-type colors, filtering, file export — built on `Dynamic<T>` so
log entries flow reactively between writers (any panel, any
backend thread) and the panel that displays them.

As of v0.4.0, lens lives in `crates/egui_lens/` inside the
egui_mobius workspace. It supersedes the older
`egui_mobius_components::event_logger`, which was built on the
signal/slot architecture and is now deprecated.

> *Implementation note:* the lens crate currently ships as a
> widget that consuming apps wrap in their own panel struct. The
> wrapper is small — a few lines — but it's an extra step that
> shouldn't be needed; lens *is* a citizen. A `LoggerCitizen`
> that implements the trait directly is on the roadmap, at which
> point the wrapper goes away. Until then, the consuming-app
> pattern in this chapter shows the wrapper.

## The shape

Two types matter: state and view.

- **`ReactiveEventLoggerState`** is the data — a `Dynamic`-wrapped
  buffer of log entries, plus filter and pagination metadata.
- **`ReactiveEventLogger<'a>`** is the view — a per-frame widget
  borrowing references to the state. You construct it inside `ui()`,
  push entries via `.log_info(...)` / `.log_warning(...)` / etc.,
  and render via `.show(ui)`.

```rust,ignore
use egui_lens::{ReactiveEventLogger, ReactiveEventLoggerState};
use egui_mobius_reactive::Dynamic;

let logger_state = Dynamic::new(ReactiveEventLoggerState::new());

// Anywhere — UI thread, backend thread, lifecycle hook:
let logger = ReactiveEventLogger::new(&logger_state);
logger.log_info("Hello, world");
logger.log_warning("Slider out of range");
logger.log_custom("network", "Connected to 192.168.1.5");

// In a panel's `ui()`:
logger.show(ui);
```

`Dynamic::clone()` is cheap (`Arc` refcount bump), so you hand
clones of `logger_state` to any thread that needs to write logs:

```rust,ignore
let state = logger_state.clone();
std::thread::spawn(move || {
    let logger = ReactiveEventLogger::new(&state);
    for i in 0..5 {
        logger.log_info(&format!("Background log #{i}"));
    }
});
```

## Custom log types and colors

Beyond the standard levels (`info`, `warning`, `error`, `debug`),
lens supports **custom typed logs** identified by string tags:

```rust,ignore
let log_colors = Dynamic::new(LogColors::default());
let mut colors = log_colors.get();
colors.set_custom_color("network", egui::Color32::from_rgb(100, 149, 237));
colors.set_custom_color("database", egui::Color32::from_rgb(106, 90, 205));
log_colors.set(colors);

// Use `with_colors` constructor to attach the color theme:
let logger = ReactiveEventLogger::with_colors(&logger_state, &log_colors);
logger.log_custom("network", "Client connected from 192.168.1.5");
logger.log_custom("database", "Inserted 5 records in 18ms");
```

Each custom type renders in its configured color. You can also set
distinct colors for the level prefix vs the message body via
`set_custom_colors(level_color, message_color)`.

## Wiring lens into a citizen panel

`ReactiveEventLogger` is a widget, not a citizen. To use it inside a
docked citizen app, wrap it in a `Citizen`-impl panel struct that
delegates rendering to the logger:

```rust,ignore
use egui_citizen::{Citizen, CitizenId, CitizenState};
use egui_lens::{ReactiveEventLogger, ReactiveEventLoggerState};

struct LoggerPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    logger_state: Dynamic<ReactiveEventLoggerState>,
}

impl Citizen for LoggerPanel {
    fn id(&self) -> &CitizenId { &self.citizen_id }
    fn citizen_state(&self) -> &CitizenState { &self.citizen_state }
    fn citizen_state_mut(&mut self) -> &mut CitizenState { &mut self.citizen_state }
}

impl LoggerPanel {
    fn show(&self, ui: &mut egui::Ui) {
        let logger = ReactiveEventLogger::new(&self.logger_state);
        logger.show(ui);
    }
}
```

The citizen lifecycle (activation, click, deactivation) is unchanged
— lens is just the rendering inside the panel.

## See also

- `examples/logger_component` — full working example (port of
  lens's `basic_custom`) demonstrating custom log types, colors,
  system info logging, and the `with_colors` constructor.
- `egui_mobius_components` — the *predecessor* logger built on
  signal/slot. Deprecated as of v0.4.0; `#[deprecated]` attributes
  surface migration warnings on use.

---

*Chapter last revised: 2026-05-03 — egui_mobius v0.4.0.*
