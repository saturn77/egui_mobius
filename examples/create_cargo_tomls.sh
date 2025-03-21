#!/bin/bash

EXAMPLES=(
    "dashboard"
    "dashboard_async"
    "reactive"
    "realtime_plot"
    "subscriber"
    "ui_refresh_events"
)

for example in "${EXAMPLES[@]}"; do
    cat > "$example/Cargo.toml" << EOL
[package]
name = "$example"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
egui_mobius = { path = "../../crates/egui_mobius" }
egui_mobius_widgets = { path = "../../crates/egui_mobius_widgets" }
egui = "0.31.1"
eframe = "0.31.1"
tokio = { version = "1", features = ["full"] }
serde = "1.0"
serde_json = "1.0"
chrono = "0.4"
rnd = "0.8"
EOL
done
