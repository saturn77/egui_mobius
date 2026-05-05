//! Reactive atom state for the 3D viewer citizen.
//!
//! Held in a `Dynamic<ReactiveViewerState>` on `ViewerCitizen`.
//! Other panels and threads observe atom toggles by reading this
//! cell; the viewer itself reads it each frame and acts on commands.

/// Shared atom state for the 3D viewer. Stored in
/// `Dynamic<ReactiveViewerState>`; consumers read or modify it to
/// observe and drive the viewer's user-facing toggles.
///
/// The `*_requested` fields are command flags — set by an atom
/// (button click, hotkey) and consumed-and-cleared by the view on
/// the next frame.
#[derive(Clone, Debug)]
pub struct ReactiveViewerState {
    /// Whether the ground grid renders.
    pub show_grid: bool,
    /// Whether the XYZ axes gizmo + screen-space labels render.
    pub show_axes: bool,
    /// Whether the measure tool is active. While true, left-drag
    /// draws a distance line on the Z=0 plane instead of orbiting.
    pub measure_active: bool,
    /// Canvas background colour as straight 8-bit RGB.
    pub background_color: [u8; 3],
    /// Set to `true` to request a view reset on the next frame —
    /// camera returns to the default tilted top-down orientation
    /// with the orbit pivot at world origin. The view consumes the
    /// flag and resets it to `false`.
    pub reset_view_requested: bool,
}

impl Default for ReactiveViewerState {
    fn default() -> Self {
        Self {
            show_grid: true,
            show_axes: true,
            measure_active: false,
            // Dark cool grey — same shade CopperForge uses behind the
            // 3D canvas; reads cleanly against both bright and dim
            // scene meshes.
            background_color: [12, 14, 20],
            reset_view_requested: false,
        }
    }
}

impl ReactiveViewerState {
    /// Build a state with default toggles.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set the grid visibility default.
    pub fn with_show_grid(mut self, show: bool) -> Self {
        self.show_grid = show;
        self
    }

    /// Builder: set the axes visibility default.
    pub fn with_show_axes(mut self, show: bool) -> Self {
        self.show_axes = show;
        self
    }

    /// Builder: set the canvas background colour.
    pub fn with_background_color(mut self, color: [u8; 3]) -> Self {
        self.background_color = color;
        self
    }
}
