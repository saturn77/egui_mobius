//! App-level messages — the intents citizens push onto their outboxes,
//! routed to the backend by the per-citizen `<citizen>_actions.rs`
//! modules. Citizen lifecycle events are separate: the registry queues
//! those and `citizens::drain_citizen` logs them.

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Settings panel asks the backend to generate a new pair of traces
    /// from the current parameters.
    Generate,
}
