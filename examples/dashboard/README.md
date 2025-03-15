# Dashboard with egui_mobius

## Overview

This example demonstrates modular communications and messages of Events and Responses within the GUI. 

```bash
cargo run -p dashboard
```

There is the ui portion that is sending events, the backend which is processing these events into Response<T> or Procssed<T> messages, and the `Dispatcher` which can register slots and send signals to anywhere in the application. 

The screenshot below shots an internal log of a "Counter Event" where the ui, backen, and `Dispatcher` are logging their actions accordingly.

![egui_mobius dashboard ](../../assets/example_dashboard.png)


## Features

- Increment and reset a numeric counter via UI buttons.
- Real-time, thread-safe logging of UI and backend events.
- Filterable logs panel (by UI, backend, Dispatcher).
- Persistent log storage (`ui_session_log.txt`).
