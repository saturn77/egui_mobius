//! App-level messages. `AppMessage::Citizen` wraps citizen lifecycle
//! events from the dispatcher; the rest are domain events the panels
//! and drain loop produce.

use egui_citizen::CitizenMessage;

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Lifecycle event from the citizen dispatcher.
    Citizen(CitizenMessage),

    /// Control panel asks the backend to run a job with the current
    /// `ParamsState` snapshot.
    Compute,

    /// Audit log — the backend slot handler has already written
    /// `last_result` / `in_flight` directly to `SharedState`; this
    /// variant fires the log line so the timing aligns with other
    /// AppMessage logging.
    BackendCompleted { value: f64, elapsed_ms: u32 },
}
