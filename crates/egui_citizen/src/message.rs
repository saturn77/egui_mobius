//! Lifecycle messages emitted by the dispatcher.
//!
//! These messages flow through [`Dispatcher::drain_messages()`](crate::Dispatcher::drain_messages)
//! and are consumed by either other panels (for reactive UI updates) or
//! backend threads (for I/O, computation, etc.).

/// A lifecycle event emitted when citizen state changes.
///
/// Route these in your update loop after `DockArea::show()`:
///
/// ```text
/// for msg in dispatcher.drain_messages() {
///     match msg {
///         CitizenMessage::Activated { id } => { /* panel became active */ }
///         CitizenMessage::Deactivated { id } => { /* panel lost focus */ }
///         _ => {}
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub enum CitizenMessage {
    /// Citizen became the active one (set).
    Activated { id: CitizenId },

    /// Citizen was deactivated (reset).
    Deactivated { id: CitizenId },

    /// Citizen was clicked this frame.
    Clicked { id: CitizenId },

    /// Citizen selection was toggled.
    Selected { id: CitizenId, selected: bool },

    /// Citizen was moved to a new dock location.
    Moved { id: CitizenId, location: [f32; 2] },

    /// Citizen visibility changed (shown or hidden).
    VisibilityChanged { id: CitizenId, visible: bool },
}

/// Unique string identifier for a citizen panel.
///
/// Panels are addressed by name — e.g., `"settings"`, `"gerber_view"`, `"plot"`.
///
/// ```rust
/// use egui_citizen::CitizenId;
///
/// let id = CitizenId::new("freq_watt");
/// assert_eq!(id.to_string(), "freq_watt");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CitizenId(pub String);

impl CitizenId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for CitizenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
