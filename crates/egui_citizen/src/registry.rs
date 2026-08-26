//! Central registry for citizen lifecycle management and message routing.

use std::collections::HashMap;

use crate::message::{CitizenId, CitizenMessage};
use crate::state::CitizenState;

/// Manages citizen registration, activation, and message routing.
///
/// The registry is the hub between the UI (panels reading shared state)
/// and the backend (threads receiving messages over channels).
///
/// # Activation
///
/// [`activate()`](Registry::activate) is the core operation — an encoded
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
///   → registry.activate("alpha")
///     → alpha.state.active = true        (reactive, immediate)
///     → beta.state.active = false
///     → queue ← [Activated, Deactivated]
///   → registry.drain_messages()
///     → route to backend threads via channels
/// ```
///
/// # Example
///
/// ```rust
/// use egui_citizen::{Registry, CitizenMessage};
///
/// let mut registry = Registry::new();
/// registry.add().with_name("alpha");
/// registry.add().with_name("beta");
///
/// registry.activate("alpha");
///
/// let messages = registry.drain_messages();
/// assert_eq!(messages.len(), 1); // Activated{alpha} only (beta was never active)
///
/// registry.activate("beta");
///
/// let messages = registry.drain_messages();
/// assert_eq!(messages.len(), 2); // Deactivated{alpha} + Activated{beta}
/// ```
pub struct Registry {
    citizens: HashMap<CitizenId, CitizenState>,
    message_queue: Vec<CitizenMessage>,
}

impl Registry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            citizens: HashMap::new(),
            message_queue: Vec::new(),
        }
    }

    /// Start adding a citizen. Finish the chain with
    /// [`with_name()`](CitizenBuilder::with_name), which registers the
    /// citizen and returns its shared [`CitizenState`] handle:
    ///
    /// ```rust
    /// # use egui_citizen::Registry;
    /// # let mut registry = Registry::new();
    /// let plot_state = registry.add().with_name("plot");
    /// ```
    ///
    /// Future per-citizen options chain between `add()` and `with_name()`.
    pub fn add(&mut self) -> CitizenBuilder<'_> {
        CitizenBuilder { registry: self }
    }

    /// Get the state of a registered citizen.
    pub fn get(&self, id: impl Into<CitizenId>) -> Option<&CitizenState> {
        self.citizens.get(&id.into())
    }

    /// Push a message onto the queue.
    ///
    /// Use this to inject messages from backend threads or from
    /// application-level logic outside the normal activation flow.
    pub fn send(&mut self, message: CitizenMessage) {
        self.message_queue.push(message);
    }

    /// Activate a citizen by name, deactivating all others.
    ///
    /// This is an encoded set/reset — exactly one citizen is active at a
    /// time. Both `Activated` and `Deactivated` messages are emitted for
    /// downstream consumers.
    pub fn activate(&mut self, id: impl Into<CitizenId>) {
        let id = id.into();
        for (cid, state) in &self.citizens {
            if *cid == id {
                state.active.set(true);
                self.message_queue
                    .push(CitizenMessage::Activated { id: cid.clone() });
            } else if state.active.get() {
                state.active.set(false);
                self.message_queue
                    .push(CitizenMessage::Deactivated { id: cid.clone() });
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

    /// Whether the registry has no citizens.
    pub fn is_empty(&self) -> bool {
        self.citizens.is_empty()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

/// In-progress citizen registration, started by [`Registry::add()`].
///
/// Chain option setters here as they grow; [`with_name()`](Self::with_name)
/// completes the registration.
pub struct CitizenBuilder<'a> {
    registry: &'a mut Registry,
}

impl CitizenBuilder<'_> {
    /// Name the citizen, completing registration.
    ///
    /// Returns the shared [`CitizenState`] handle. The handle can be cloned
    /// and given to the panel struct — all clones share the same underlying
    /// `Dynamic<T>` fields, so changes made by the registry are visible to
    /// the panel immediately.
    pub fn with_name(self, name: impl Into<CitizenId>) -> CitizenState {
        let state = CitizenState::new();
        self.registry.citizens.insert(name.into(), state.clone());
        state
    }
}
