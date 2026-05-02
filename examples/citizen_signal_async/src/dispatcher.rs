//! Dispatcher wiring: register citizens at startup, drain lifecycle
//! messages each frame, route AppMessage events to the work signal.
//!
//! Mirrors `filter_plotter::dispatcher` deliberately. The difference is
//! that the work boundary here is a `Signal<WorkRequest>` (cross-thread,
//! async via `egui_mobius`'s signal/slot bus) rather than a synchronous
//! `BackendKind` trait object run inline on the UI thread.

use egui_citizen::{CitizenMessage, Dispatcher};
use egui_mobius::Signal;
use egui_mobius_reactive::Dynamic;

use crate::messages::AppMessage;
use crate::state::{SharedState, WorkRequest};

/// Drain citizen lifecycle messages and append them to the shared log.
/// Call once per frame after `DockArea::show`.
pub fn drain_citizen(dispatcher: &mut Dispatcher, log: &Dynamic<Vec<String>>) {
    for msg in dispatcher.drain_messages() {
        append_log(log, format_citizen(&msg));
    }
}

/// Route an app-level message. `Compute` snapshots the params and
/// pushes a `WorkRequest` onto the signal bus; the backend slot picks
/// it up off the UI thread. `BackendCompleted` logs the completion —
/// the actual result has already been written to `state.last_result`
/// by the slot handler that received the response.
pub fn handle(
    msg: AppMessage,
    state: &SharedState,
    work_signal: &Signal<WorkRequest>,
    log: &Dynamic<Vec<String>>,
) {
    match msg {
        AppMessage::Compute => {
            let req = state.params.snapshot();
            state.in_flight.set(true);
            append_log(
                log,
                format!(
                    "[ui] submit: duration_ms={}, seed={:.4}",
                    req.duration_ms, req.seed,
                ),
            );
            if let Err(e) = work_signal.send(req) {
                append_log(log, format!("[ui] send failed: {e}"));
                state.in_flight.set(false);
            }
        }
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
