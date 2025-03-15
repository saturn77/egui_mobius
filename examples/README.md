# Examples in `egui_mobius`

The highlighted examples are the core projects that are actively maintained and intended to showcase the key features of `egui_mobius`.

These examples remain in the `main` branch and serve as reference implementations for different patterns such as async integration, real-time UI updates, and the message/event system.

There are several original and older examples in this `deprecated` branch. 

---

## ✅ Highlighted Examples

### `dashboard`
Demonstrates how to use the `Dispatcher` to register
slots and send responses, and has internal logging to
show what ui (events) , backend processing, and those
messages handled by the `Dispatcher.` 

- A good place to start to understand the separation of ui and backend concerns within the overall framework of having a `Dispatcher.
- Logging overal GUI events and processed responses, may be useful when making a GUI template from which to build other applications
- A more modular example than some of the other examples based on the utility of the `Dispatcher`
---

### `dashboard_async`
An extension of the `dashboard` example that integrates asynchronous tasks using a `tokio` runtime.

- Demonstrates async background tasks (e.g., fetching data from APIs)
- Ideal reference for combining `egui_mobius` with long-running processes

---
### `ui_refresh_events`
Demonstrates how to trigger UI refreshes based on custom timed or programmatic events. There are several widgets
involved in this application, and demonstrates how sending
events for each widget is done. 

- Useful when getting started, does not have a true `Dispatcher` instance inside of it
- Showcases the `RequestRepaint` integration pattern
- Useful for building more basic applications

---

### `realtime_plot`
Visualizes dynamic data updates in real time using line charts.

- Demonstrates how to push streaming data into the UI
- Useful for monitoring dashboards, telemetry, or charts

---

## 📝 Notes

- Other, more minimal or legacy examples have been moved to the `deprecated` branch.
- Dev/test utilities will be in `examples/dev/`.

---

Feel free to explore these examples when building your own app or library with `egui_mobius`. They cover a range of patterns and are kept up to date with the latest APIs.
