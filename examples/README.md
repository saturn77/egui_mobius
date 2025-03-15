# Examples in `egui_mobius`

The highlighted examples are the core projects that are actively maintained and intended to showcase the key features of `egui_mobius`.

These examples remain in the `main` branch and serve as reference implementations for different patterns such as async integration, real-time UI updates, and the message/event system.

---

## ✅ Highlighted Examples

### `dashboard`
A feature-rich example combining multiple widgets and internal state management. Demonstrates how to build a cohesive, multi-panel UI with `egui_mobius`.

- Shows event handling and component messaging
- Integrates multiple UI features

---

### `dashboard_async`
An extension of the `dashboard` example that integrates asynchronous tasks using a `tokio` runtime.

- Demonstrates async background tasks (e.g., fetching data from APIs)
- Ideal reference for combining `egui_mobius` with long-running processes

---

### `ui_refresh_events`
Demonstrates how to trigger UI refreshes based on custom timed or programmatic events.

- Showcases the `RequestRepaint` integration pattern
- Useful for streaming or polling-style apps

---

### `realtime_plot`
Visualizes dynamic data updates in real time using line charts.

- Demonstrates how to push streaming data into the UI
- Useful for monitoring dashboards, telemetry, or charts

---

## 📝 Notes

- Other, more minimal or legacy examples have been moved to the [`deprecated`](https://github.com/YOUR-ORG/egui_mobius/tree/deprecated) branch.
- Dev/test utilities have been relocated to `examples/dev/`.

---

Feel free to explore these examples when building your own app or library with `egui_mobius`. They cover a range of patterns and are kept up to date with the latest APIs.
