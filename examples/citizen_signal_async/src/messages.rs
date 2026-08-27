//! App-level messages — the intents citizens push onto their outboxes,
//! routed to the backend signal by the per-citizen
//! `<citizen>_actions.rs` modules. Citizen lifecycle events are
//! separate: the registry queues those and `citizens::drain_citizen`
//! logs them.

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Control panel asks the backend to run a job with the current
    /// `ParamsState` snapshot.
    Compute,
}
