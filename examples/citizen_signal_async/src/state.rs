//! Shared application state and the boundary types crossing the work
//! signal/slot bus.

use egui_mobius_reactive::Dynamic;

/// Reactive parameters edited by the Control panel. Snapshotted into a
/// `WorkRequest` at Compute time so the backend gets a stable owned
/// value — no reactivity crosses the work boundary.
pub struct ParamsState {
    pub work_duration_ms: Dynamic<u32>,
    pub seed: Dynamic<f64>,
}

impl ParamsState {
    pub fn defaults() -> Self {
        Self {
            work_duration_ms: Dynamic::new(500),
            seed: Dynamic::new(0.25),
        }
    }

    pub fn snapshot(&self) -> WorkRequest {
        WorkRequest {
            duration_ms: self.work_duration_ms.get(),
            seed: self.seed.get(),
        }
    }
}

/// Everything the panels read from. Control reads/writes `params`,
/// Result reads `last_result` and `in_flight`, Logger reads `log`.
pub struct SharedState {
    pub params: ParamsState,
    pub last_result: Dynamic<f64>,
    pub in_flight: Dynamic<bool>,
    pub log: Dynamic<Vec<String>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            params: ParamsState::defaults(),
            last_result: Dynamic::new(0.0),
            in_flight: Dynamic::new(false),
            log: Dynamic::new(Vec::new()),
        }
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}

// ── Boundary types — values crossing the signal/slot bus ─────────────
//
// These travel through the signal bus to the async backend and back.
// `Send + 'static + Copy` so they ride the channel without lifetime
// concerns. They will likely move to `backend.rs` in Phase 2 alongside
// the async work function itself.

#[derive(Debug, Clone, Copy)]
pub struct WorkRequest {
    pub duration_ms: u32,
    pub seed: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct WorkResponse {
    pub value: f64,
    pub elapsed_ms: u32,
}
