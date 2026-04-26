# egui_citizen

First-class dock panel lifecycle and state tracking for egui.

## The Problem

In egui's immediate mode, dock panels have no persistent identity. When multiple
panels are visible simultaneously (e.g., undocked into separate `egui_dock` nodes),
there is no built-in way to know which panel the user last interacted with. This
leads to per-frame state races — whichever panel renders last wins.

## The Solution

**Each panel of a dockable egui application should implement the `Citizen` trait.**

A `Citizen` is a dock panel with persistent identity and lifecycle state that
survives across frames. State changes are dispatched as messages through an
Elm-style update loop, eliminating frame-order dependencies entirely.

## Core Types

| Type | Purpose |
|------|---------|
| `Citizen` | Trait implemented by each dock panel to declare identity and respond to lifecycle events |
| `CitizenState` | Reactive state (`Dynamic<T>`) tracking: active, clicked, selected, moved, location, visible |
| `CitizenMessage` | Lifecycle events: Activated, Deactivated, Clicked, Selected, Moved, VisibilityChanged |
| `CitizenRegistry` | Central registry managing all citizen panels and dispatching messages |
| `CitizenId` | Unique string identifier for a citizen panel |

## Message Flow

```
┌─────────────┐    on_tab_button     ┌──────────────────┐
│  Tab Header  │ ──── clicked() ───> │ CitizenRegistry   │
│  (egui_dock) │                     │   .activate(id)   │
└─────────────┘                      └────────┬─────────┘
                                              │
                                    drain_messages()
                                              │
                              ┌───────────────┴───────────────┐
                              ▼                               ▼
                     Another Citizen                  Backend Dispatcher
                  (e.g., Plot panel                  (e.g., service layer
                   switches curve)                    triggers computation)
```

## Usage

```rust
use egui_citizen::{Citizen, CitizenId, CitizenRegistry, CitizenState};

// 1. Register citizens at app startup
let mut registry = CitizenRegistry::new();
registry.register(CitizenId::new("freq_watt"));
registry.register(CitizenId::new("volt_watt"));
registry.register(CitizenId::new("volt_var"));

// 2. In your TabViewer, activate on click
fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
    if response.clicked() {
        if let Some(id) = tab.citizen_id() {
            self.registry.activate(&id);  // one-hot: one active, rest deactivated
        }
    }
}

// 3. Consumers react to messages
for msg in registry.drain_messages() {
    match msg {
        CitizenMessage::Activated { id } => { /* update plot, notify backend */ }
        CitizenMessage::Deactivated { id } => { /* cleanup */ }
        _ => {}
    }
}
```

## Design Principles

- **A panel is the citizen.** Widgets inside a panel are implementation details —
  they don't need their own lifecycle tracking.
- **Messages have two consumer types:** other citizens (peer-to-peer) or the
  backend dispatcher (service layer).
- **`activate()` is an encoded set/reset.** Exactly one citizen in the registry is active
  at a time. Activating one deactivates all others — like an encoded set/reset.
- **Two things make rugged apps:** dockable widgets and threading. Citizen provides
  the lifecycle layer that bridges these two fundamentals.

## Examples

- `examples/citizen_dock/` — basic demo: three algo tabs, reactive plot, message logger
- `examples/serial_plotter/` — real-time serial plotter with live hardware (RP2350)
