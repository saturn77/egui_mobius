//! Actions for the control citizen: its outbox messages, routed to the
//! backend work signal. This is the dispatch path — the only place the
//! word applies. Mirrors `filter_plotter::settings_actions`
//! deliberately; the difference is that the work boundary here is a
//! `Signal<WorkRequest>` (cross-thread, async via `egui_mobius`'s
//! signal/slot bus) rather than a synchronous `BackendKind` trait
//! object run inline on the UI thread.

use egui_mobius::Signal;
use egui_mobius_reactive::Dynamic;

use crate::citizens::append_log;
use crate::messages::AppMessage;
use crate::state::{SharedState, WorkRequest};

/// Route one of the control citizen's app-level messages. `Compute`
/// snapshots the params and pushes a `WorkRequest` onto the signal bus;
/// the backend slot picks it up off the UI thread.
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
