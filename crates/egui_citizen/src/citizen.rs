//! The Citizen trait — persistent identity and lifecycle for dock panels.

use crate::message::CitizenId;
use crate::state::CitizenState;

/// A dock panel with persistent identity and lifecycle state.
///
/// Implement this on each panel struct to give it a name, reactive state,
/// and lifecycle hooks. The [`Dispatcher`](crate::Dispatcher) manages
/// activation and message dispatch across all citizens.
///
/// # Minimal implementation
///
/// ```rust,ignore
/// use egui_citizen::{Citizen, CitizenId, CitizenState};
///
/// struct PlotPanel {
///     citizen_id: CitizenId,
///     citizen_state: CitizenState,
/// }
///
/// impl PlotPanel {
///     fn new(state: CitizenState) -> Self {
///         Self { citizen_id: CitizenId::new("plot"), citizen_state: state }
///     }
/// }
///
/// impl Citizen for PlotPanel {
///     fn id(&self) -> &CitizenId { &self.citizen_id }
///     fn citizen_state(&self) -> &CitizenState { &self.citizen_state }
///     fn citizen_state_mut(&mut self) -> &mut CitizenState { &mut self.citizen_state }
/// }
/// ```
///
/// # Lifecycle hooks
///
/// Override `on_activate()`, `on_deactivate()`, or `on_click()` to run
/// custom logic when the citizen's state changes. The defaults just update
/// the reactive `CitizenState` fields.
pub trait Citizen {
    /// This citizen's unique identifier.
    fn id(&self) -> &CitizenId;

    /// Read-only access to lifecycle state.
    fn citizen_state(&self) -> &CitizenState;

    /// Mutable access to lifecycle state.
    fn citizen_state_mut(&mut self) -> &mut CitizenState;

    /// Called when this citizen becomes the active one.
    fn on_activate(&mut self) {
        self.citizen_state_mut().active.set(true);
    }

    /// Called when this citizen is deactivated.
    fn on_deactivate(&mut self) {
        self.citizen_state_mut().active.set(false);
    }

    /// Called when this citizen is clicked.
    fn on_click(&mut self) {
        self.citizen_state_mut().clicked.set(true);
    }

    /// Whether this citizen is currently active.
    fn is_active(&self) -> bool {
        self.citizen_state().active.get()
    }

    /// Whether this citizen is currently selected.
    fn is_selected(&self) -> bool {
        self.citizen_state().selected.get()
    }
}
