//! `CanvasCitizen` — the dock-panel integration that hosts a [`Scene`]
//! inside an `egui_mobius` application.
//!
//! A `CanvasCitizen` owns a reactive `Dynamic<Scene>` so that property
//! panels, palette panels, and the canvas view itself all see edits
//! atomically and re-render only what changed.
//!
//! Not yet implemented — sketched API:
//!
//! ```ignore
//! pub struct CanvasCitizen {
//!     pub id: CitizenId,
//!     pub scene: Dynamic<Scene>,
//!     pub selection: Dynamic<Selection>,
//!     // routing strategy, undo stack, palette, etc.
//! }
//!
//! impl Citizen for CanvasCitizen { ... }
//! ```

use crate::model::Scene;
use egui_mobius_reactive::Dynamic;

/// Reactive container for the canvas a citizen panel renders.
pub struct CanvasCitizen {
    pub scene: Dynamic<Scene>,
}

impl CanvasCitizen {
    pub fn new(scene: Scene) -> Self {
        Self {
            scene: Dynamic::new(scene),
        }
    }
}
