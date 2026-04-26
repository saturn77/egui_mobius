//! Reactive lifecycle state for a citizen panel.
//!
//! Each field is a [`Dynamic<T>`](egui_mobius_reactive::Dynamic) — other panels
//! and threads can read the value without polling. When the dispatcher calls
//! `activate()`, the `active` field updates immediately and all readers see
//! the new value on their next `.get()`.

use egui_mobius_reactive::Dynamic;

/// Per-citizen lifecycle state.
///
/// All fields are reactive. Clone this struct to share it — clones point
/// to the same underlying `Dynamic<T>` storage.
///
/// ```rust
/// use egui_citizen::CitizenState;
///
/// let state = CitizenState::new();
/// let clone = state.clone();
///
/// state.active.set(true);
/// assert!(clone.active.get()); // same underlying value
/// ```
#[derive(Clone)]
pub struct CitizenState {
    /// Whether this citizen is the active one in its group.
    pub active: Dynamic<bool>,

    /// True during the frame the citizen was clicked.
    pub clicked: Dynamic<bool>,

    /// Persistent selection toggle (independent of activation).
    pub selected: Dynamic<bool>,

    /// True if the citizen has been moved to a new dock location.
    pub moved: Dynamic<bool>,

    /// Current position in the dock layout, if tracked.
    pub location: Dynamic<[f32; 2]>,

    /// Whether the citizen is currently visible.
    pub visible: Dynamic<bool>,
}

impl CitizenState {
    pub fn new() -> Self {
        Self {
            active: Dynamic::new(false),
            clicked: Dynamic::new(false),
            selected: Dynamic::new(false),
            moved: Dynamic::new(false),
            location: Dynamic::new([0.0, 0.0]),
            visible: Dynamic::new(false),
        }
    }
}

impl Default for CitizenState {
    fn default() -> Self {
        Self::new()
    }
}
