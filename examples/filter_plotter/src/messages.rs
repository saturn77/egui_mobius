//! App-level messages. `AppMessage::Citizen` wraps the citizen lifecycle
//! events from the dispatcher; the rest are domain events the panels and
//! drain loop produce.

use egui_citizen::CitizenMessage;

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Lifecycle event from the citizen dispatcher.
    Citizen(CitizenMessage),

    /// Settings panel asks the backend to generate a new pair of traces
    /// from the current parameters.
    Generate,

    /// Backend reports a finished run. (Used here as a log signal —
    /// the traces themselves are written to SharedState directly.)
    GenerateCompleted { samples: usize },
}
