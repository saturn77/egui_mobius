//! The citizens of the app plus the registry wiring they share:
//! the lifecycle drain and the bounded log helper. Everything here is
//! registry-side; routing a citizen's outbox messages to the backend
//! signal lives in `src/<citizen>_actions.rs`.

pub mod control;
pub mod logger;
pub mod result;

use egui_citizen::{CitizenMessage, Registry};
use egui_mobius_reactive::Dynamic;

/// Drain citizen lifecycle messages and append them to the shared log.
/// Call once per frame after `DockArea::show`.
pub fn drain_citizen(registry: &mut Registry, log: &Dynamic<Vec<String>>) {
    for msg in registry.drain_messages() {
        append_log(log, format_citizen(&msg));
    }
}

const MAX_LOG_LINES: usize = 500;

/// Append a line to the log, capped at `MAX_LOG_LINES` so memory stays
/// bounded over long sessions.
pub fn append_log(log: &Dynamic<Vec<String>>, line: String) {
    let mut buf = log.get();
    buf.push(line);
    if buf.len() > MAX_LOG_LINES {
        let drop = buf.len() - MAX_LOG_LINES;
        buf.drain(0..drop);
    }
    log.set(buf);
}

fn format_citizen(msg: &CitizenMessage) -> String {
    match msg {
        CitizenMessage::Activated { id } => format!("[citizen] {} activated", id),
        CitizenMessage::Deactivated { id } => format!("[citizen] {} deactivated", id),
        CitizenMessage::Clicked { id } => format!("[citizen] {} clicked", id),
        CitizenMessage::Selected { id, selected } => {
            format!("[citizen] {} selected={}", id, selected)
        }
        CitizenMessage::Moved { id, location } => format!(
            "[citizen] {} moved to [{:.1}, {:.1}]",
            id, location[0], location[1]
        ),
        CitizenMessage::VisibilityChanged { id, visible } => {
            format!("[citizen] {} visible={}", id, visible)
        }
    }
}
