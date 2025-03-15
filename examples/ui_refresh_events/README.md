# UI Refresh Events Example – `egui_mobius`

This example demonstrates an efficient, event-driven architecture using **signals and slots** for real-time UI updates in a multithreaded context. A minimal but powerful demonstration of **frontend-backend separation** using `egui_mobius`. 

```bash
cargo run -p ui_refresh_events
```

## Features

- Real-time slider updates
- Button-triggered backend events (`Foo`, `Bar`)
- ComboBox-driven actions
- Scrollable log panel with backend/UI entries
- Application command dispatching (`OS Info`, `Clear Logger`, etc.)
- Event detection via `Edge<T>` to avoid redundant updates


## Architecture

- **UI (`UiApp`)**: 
  - Sends events using `Signal<EventType>`
  - Receives processed results via `Slot<ProcessedType>`
  - UI updates only when backend sends new data

- **Backend Thread**: 
  - Listens to `Slot<EventType>`
  - Processes events and sends responses via `Signal<ProcessedType>`
  - Manages application logic (buttons, slider, combo)



