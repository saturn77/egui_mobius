//! Central dispatcher for citizen lifecycle management and message routing.

use std::collections::HashMap;

use crate::message::{CitizenId, CitizenMessage};
use crate::state::CitizenState;

/// Manages citizen registration, activation, and message dispatch.
///
/// The dispatcher is the hub between the UI (panels reading shared state)
/// and the backend (threads receiving messages over channels).
///
/// # Activation
///
/// [`activate()`](Dispatcher::activate) is the core operation — an encoded
/// set/reset. When you activate citizen "alpha":
/// - `alpha.active` is set to `true`
/// - All other active citizens are set to `false`
/// - `Activated { id: "alpha" }` and `Deactivated { id: "beta" }` messages
///   are pushed to the queue
///
/// # Message flow
///
/// ```text
/// Tab click
///   → dispatcher.activate("alpha")
///     → alpha.state.active = true        (reactive, immediate)
///     → beta.state.active = false
///     → queue ← [Activated, Deactivated]
///   → dispatcher.drain_messages()
///     → route to backend threads via channels
/// ```
///
/// # Example
///
/// ```rust
/// use egui_citizen::{Dispatcher, CitizenId, CitizenMessage};
///
/// let mut dispatcher = Dispatcher::new();
/// dispatcher.register(CitizenId::new("alpha"));
/// dispatcher.register(CitizenId::new("beta"));
///
/// dispatcher.activate(&CitizenId::new("alpha"));
///
/// let messages = dispatcher.drain_messages();
/// assert_eq!(messages.len(), 1); // Activated{alpha} only (beta was never active)
///
/// dispatcher.activate(&CitizenId::new("beta"));
///
/// let messages = dispatcher.drain_messages();
/// assert_eq!(messages.len(), 2); // Deactivated{alpha} + Activated{beta}
/// ```
pub struct Dispatcher {
    citizens: HashMap<CitizenId, CitizenState>,
    message_queue: Vec<CitizenMessage>,
}

impl Dispatcher {
    /// Create an empty dispatcher.
    pub fn new() -> Self {
        Self {
            citizens: HashMap::new(),
            message_queue: Vec::new(),
        }
    }

    /// Register a citizen and return its shared state handle.
    ///
    /// The returned [`CitizenState`] can be cloned and handed to the panel
    /// struct. All clones share the same underlying `Dynamic<T>` fields,
    /// so changes made by the dispatcher are visible to the panel immediately.
    pub fn register(&mut self, id: CitizenId) -> CitizenState {
        let state = CitizenState::new();
        self.citizens.insert(id, state.clone());
        state
    }

    /// Get the state of a registered citizen.
    pub fn get(&self, id: &CitizenId) -> Option<&CitizenState> {
        self.citizens.get(id)
    }

    /// Push a message onto the queue.
    ///
    /// Use this to inject messages from backend threads or from
    /// application-level logic outside the normal activation flow.
    pub fn send(&mut self, message: CitizenMessage) {
        self.message_queue.push(message);
    }

    /// Activate a citizen by ID, deactivating all others.
    ///
    /// This is an encoded set/reset — exactly one citizen is active at a
    /// time. Both `Activated` and `Deactivated` messages are emitted for
    /// downstream consumers.
    pub fn activate(&mut self, id: &CitizenId) {
        for (cid, state) in &self.citizens {
            if cid == id {
                state.active.set(true);
                self.message_queue.push(CitizenMessage::Activated { id: cid.clone() });
            } else if state.active.get() {
                state.active.set(false);
                self.message_queue.push(CitizenMessage::Deactivated { id: cid.clone() });
            }
        }
    }

    /// Drain all pending messages, returning them for processing.
    ///
    /// Call this once per frame after `DockArea::show()` returns.
    /// Messages are consumed — calling again returns an empty vec
    /// until new messages are produced.
    pub fn drain_messages(&mut self) -> Vec<CitizenMessage> {
        std::mem::take(&mut self.message_queue)
    }

    /// Number of registered citizens.
    pub fn len(&self) -> usize {
        self.citizens.len()
    }

    /// Whether the dispatcher has no citizens.
    pub fn is_empty(&self) -> bool {
        self.citizens.is_empty()
    }
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self::new()
    }
}
