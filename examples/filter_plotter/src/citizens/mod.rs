//! The citizens of the app — one module per citizen — plus the registry
//! wiring they share: registration ids, the lifecycle drain, and log
//! helpers. Everything here is registry-side; routing a citizen's
//! outbox messages to the backend lives in `src/<citizen>_actions.rs`.

pub mod editor;
pub mod logger;
pub mod plot;
pub mod settings;

use egui_citizen::{CitizenMessage, Registry};
use egui_lens::ReactiveEventLogger;

use crate::state::SharedState;

/// Drain citizen lifecycle messages from the registry and route them
/// into the shared lens-backed log. Call once per frame after
/// `DockArea::show`.
pub fn drain_citizen(registry: &mut Registry, state: &SharedState) {
    let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
    for msg in registry.drain_messages() {
        logger.log_custom("citizen", &format_citizen(&msg));
    }
}

/// Append a single info-level line to the log. Convenience wrapper for
/// places that want a one-liner without constructing a logger.
pub fn append_log(state: &SharedState, line: String) {
    let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
    logger.log_info(&line);
}

fn format_citizen(msg: &CitizenMessage) -> String {
    match msg {
        CitizenMessage::Activated { id } => format!("{} activated", id),
        CitizenMessage::Deactivated { id } => format!("{} deactivated", id),
        CitizenMessage::Clicked { id } => format!("{} clicked", id),
        CitizenMessage::Selected { id, selected } => {
            format!("{} selected={}", id, selected)
        }
        CitizenMessage::Moved { id, location } => format!(
            "{} moved to [{:.1}, {:.1}]",
            id, location[0], location[1]
        ),
        CitizenMessage::VisibilityChanged { id, visible } => {
            format!("{} visible={}", id, visible)
        }
    }
}
