//! App-level messages. `AppMessage::Citizen` wraps the citizen lifecycle
//! events from the dispatcher; the rest are domain events the panels and
//! drain loop produce.

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Settings panel asks the backend to generate a new pair of traces
    /// from the current parameters.
    Generate,
}
