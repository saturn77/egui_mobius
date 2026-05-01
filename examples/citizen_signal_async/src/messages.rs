//! App-level messages. `AppMessage::Citizen` wraps citizen lifecycle
//! events from the dispatcher; the rest are domain events the panels
//! and drain loop produce.

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Control panel asks the backend to run a job with the current
    /// `ParamsState` snapshot.
    Compute,
}
