//! Dispatcher wiring: register citizens at startup, drain lifecycle
//! messages each frame, route AppMessage events.
//!
//! Centralizing this in one module keeps main.rs to its job (eframe
//! shell + dock layout) and gives the dispatcher code one place to
//! evolve as the app grows.

use egui_citizen::{CitizenMessage, Dispatcher};
use egui_lens::{ReactiveEventLogger, ReactiveEventLoggerState};
use egui_mobius_reactive::Dynamic;

use crate::backend::BackendKind;
use crate::messages::AppMessage;
use crate::state::SharedState;

/// Drain citizen lifecycle messages from the dispatcher and route them
/// into the shared lens-backed log. Call once per frame after
/// `DockArea::show`.
pub fn drain_citizen(dispatcher: &mut Dispatcher, log: &Dynamic<ReactiveEventLoggerState>) {
    let logger = ReactiveEventLogger::new(log);
    for msg in dispatcher.drain_messages() {
        logger.log_custom("citizen", &format_citizen(&msg));
    }
}

/// Route an app-level message. `Generate` runs the backend synchronously
/// and stores the resulting traces in shared state; the others log.
pub fn handle<B>(
    msg: AppMessage,
    state: &SharedState,
    backend: &mut B,
    log: &Dynamic<ReactiveEventLoggerState>,
) where
    B: BackendKind<Sample = f32>,
{
    let logger = ReactiveEventLogger::new(log);
    match msg {
        AppMessage::Generate => {
            let params = state.params.snapshot();
            let traces = backend.run(&params);
            let n = traces.input.len();
            state.traces.set(traces);
            logger.log_info(&format!(
                "backend ({}) produced {} samples",
                backend.name(),
                n
            ));
        }
    }
}

/// Append a single info-level line to the log. Convenience wrapper for
/// places that want a one-liner without constructing a logger.
pub fn append_log(log: &Dynamic<ReactiveEventLoggerState>, line: String) {
    let logger = ReactiveEventLogger::new(log);
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
