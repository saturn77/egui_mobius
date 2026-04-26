# Wiring into egui_dock

> **Stub.** Adapt from `examples/citizen_dock/`.

Implement `TabViewer`, route `on_tab_button` into
`dispatcher.activate()`, drain messages once after `DockArea::show()`.

The key insight: **the dispatcher doesn't know about dock at all.**
That boundary is user code, by design — egui-citizen has no dependency
on `egui_dock`.
