//! Actions for the settings citizen: its outbox messages, routed to
//! the backend. This is the dispatch path — the only place the word
//! applies. One `<citizen>_actions.rs` per citizen that emits
//! `AppMessage`s; citizens that only render need none.

use crate::backend::BackendKind;
use crate::citizens::append_log;
use crate::messages::AppMessage;
use crate::state::SharedState;

use egui_lens::ReactiveEventLogger;

/// Route one of the settings citizen's app-level messages. `Generate`
/// runs the backend synchronously and stores the resulting traces in
/// shared state.
pub fn handle<B>(msg: AppMessage, state: &SharedState, backend: &mut B)
where
    B: BackendKind<Sample = f32>,
{
    let _ = &append_log; // module-local convenience re-export kept nearby
    let logger = ReactiveEventLogger::with_colors(&state.log, &state.log_colors);
    match msg {
        AppMessage::Generate => {
            let params = state.params.snapshot();
            let traces = backend.run(&params);
            let n = traces.input.len();
            state.traces.set(traces);
            logger.log_custom(
                "backend",
                &format!("{} produced {} samples", backend.name(), n),
            );
        }
    }
}
