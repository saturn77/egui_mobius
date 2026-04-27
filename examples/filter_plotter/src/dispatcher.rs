//! Dispatcher wiring: register citizens at startup, drain lifecycle
//! messages each frame, route AppMessage events.
//!
//! Centralizing this in one module keeps main.rs to its job (eframe
//! shell + dock layout) and gives the dispatcher code one place to
//! evolve as the app grows.

use egui_citizen::{CitizenId, CitizenMessage, CitizenState, Dispatcher};
use egui_mobius_reactive::Dynamic;

use crate::backend::BackendKind;
use crate::messages::AppMessage;
use crate::state::SharedState;
use crate::tabs::{PLOT_ID, SETTINGS_ID, TERMINAL_ID};

/// The three CitizenStates the app's panels need to hold, kept in a
/// single struct so the App doesn't have to thread three values out of
/// `register_citizens`.
pub struct RegisteredCitizens {
    pub plot:     CitizenState,
    pub settings: CitizenState,
    pub terminal: CitizenState,
}

/// Register the three citizens with the dispatcher and activate `plot`
/// as the default focus. Returns the panel-bound state handles.
pub fn register_citizens(dispatcher: &mut Dispatcher) -> RegisteredCitizens {
    let plot     = dispatcher.register(CitizenId::new(PLOT_ID));
    let settings = dispatcher.register(CitizenId::new(SETTINGS_ID));
    let terminal = dispatcher.register(CitizenId::new(TERMINAL_ID));

    dispatcher.activate(&CitizenId::new(PLOT_ID));

    RegisteredCitizens { plot, settings, terminal }
}

/// Drain citizen lifecycle messages from the dispatcher and append them
/// to the shared log. Call once per frame after `DockArea::show`.
pub fn drain_citizen(dispatcher: &mut Dispatcher, log: &Dynamic<Vec<String>>) {
    for msg in dispatcher.drain_messages() {
        append_log(log, format_citizen(&msg));
    }
}

/// Route an app-level message. `Generate` runs the backend synchronously
/// and stores the resulting traces in shared state; the others log.
pub fn handle<B: BackendKind>(
    msg: AppMessage,
    state: &SharedState,
    backend: &mut B,
    log: &Dynamic<Vec<String>>,
) {
    match msg {
        // Already drained directly via `drain_citizen`.
        AppMessage::Citizen(_) => {}

        AppMessage::Generate => {
            let params = state.params.snapshot();
            let traces = backend.run(&params);
            let n = traces.input.len();
            state.traces.set(traces);
            append_log(
                log,
                format!(
                    "[INFO] backend ({}) produced {} samples",
                    backend.name(),
                    n,
                ),
            );
        }

        AppMessage::GenerateCompleted { samples } => {
            append_log(log, format!("[INFO] generate completed: {} samples", samples));
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
        CitizenMessage::Activated   { id } => format!("[citizen] {} activated",   id),
        CitizenMessage::Deactivated { id } => format!("[citizen] {} deactivated", id),
        CitizenMessage::Clicked     { id } => format!("[citizen] {} clicked",     id),
        CitizenMessage::Selected    { id, selected } =>
            format!("[citizen] {} selected={}", id, selected),
        CitizenMessage::Moved       { id, location } =>
            format!("[citizen] {} moved to [{:.1}, {:.1}]", id, location[0], location[1]),
        CitizenMessage::VisibilityChanged { id, visible } =>
            format!("[citizen] {} visible={}", id, visible),
    }
}
