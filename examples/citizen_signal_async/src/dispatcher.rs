//! Dispatcher wiring: register citizens at startup, drain lifecycle
//! messages each frame, route AppMessage events to the work signal.
//!
//! Mirrors `filter_plotter::dispatcher` deliberately. The difference is
//! that the work boundary here is a `Signal<WorkRequest>` (cross-thread,
//! async via `egui_mobius`'s signal/slot bus) rather than a synchronous
//! `BackendKind` trait object run inline on the UI thread.

use egui_citizen::{CitizenId, CitizenMessage, CitizenState, Dispatcher};
use egui_mobius::Signal;
use egui_mobius_reactive::Dynamic;

use crate::messages::AppMessage;
use crate::state::{SharedState, WorkRequest};
use crate::tabs::{CONTROL_ID, LOGGER_ID, RESULT_ID};

/// The three CitizenStates the panels need to hold, kept together so
/// `App::new()` doesn't have to thread three values out of
/// `register_citizens`.
pub struct RegisteredCitizens {
    pub control: CitizenState,
    pub result: CitizenState,
    pub logger: CitizenState,
}

/// Register the three citizens with the dispatcher and activate
/// `control` as the default focus. Returns the panel-bound state
/// handles.
pub fn register_citizens(dispatcher: &mut Dispatcher) -> RegisteredCitizens {
    let control = dispatcher.register(CitizenId::new(CONTROL_ID));
    let result = dispatcher.register(CitizenId::new(RESULT_ID));
    let logger = dispatcher.register(CitizenId::new(LOGGER_ID));

    dispatcher.activate(&CitizenId::new(CONTROL_ID));

    RegisteredCitizens {
        control,
        result,
        logger,
    }
}

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
        // Already drained directly via `drain_citizen`.
        AppMessage::Citizen(_) => {}

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

        AppMessage::BackendCompleted { value, elapsed_ms } => {
            append_log(
                log,
                format!(
                    "[ui] backend completed: value={:.4} elapsed_ms={}",
                    value, elapsed_ms,
                ),
            );
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
