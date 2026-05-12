use nalgebra::{Matrix4, Perspective3, Translation3, UnitQuaternion, Vector3, Vector4};

/// Orbit camera: rotates the world about `target`, viewed from `zoom` units
/// along -Z in camera space. `target` lets zoom-to-region + board-recenter
/// move the orbit pivot to any scene point instead of being stuck at origin.
pub struct Camera {
    pub rotation: UnitQuaternion<f32>,
    pub zoom: f32,
    pub target: Vector3<f32>,
}

impl Default for Camera {
    fn default() -> Self {
        // Tilt the world ~55° forward so the XY plane reads as a receding
        // floor rather than edge-on, and pull the camera back far enough to
        // see a 10×10 ground grid before any board is loaded.
        let tilt = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), -55f32.to_radians());
        Self {
            rotation: tilt,
            zoom: 12.0,
            target: Vector3::zeros(),
        }
    }
}

impl Camera {
    /// Build `P * V` (identity model). The view matrix translates the world
    /// so `target` lands at camera-space origin, rotates, then pulls back by
    /// `zoom` — orbit pivots on `target`, not on the world origin.
    pub fn mvp(&self, viewport: egui::Rect) -> Matrix4<f32> {
        let aspect = (viewport.width() / viewport.height().max(1.0)).max(0.01);
        let proj = Perspective3::new(aspect, 60f32.to_radians(), 0.1, 10_000.0);
        let view = Translation3::new(0.0, 0.0, -self.zoom).to_homogeneous()
            * self.rotation.to_homogeneous()
            * Translation3::from(-self.target).to_homogeneous();
        proj.as_matrix() * view
    }

    /// Orbit by a screen-space drag delta (pixels). Yaw about world Y,
    /// pitch about world X.
    pub fn orbit(&mut self, drag_delta: egui::Vec2) {
        let yaw = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), drag_delta.x * 0.01);
        let pitch = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), drag_delta.y * 0.01);
        self.rotation = yaw * pitch * self.rotation;
    }

    /// Multiplicative zoom: >1 → closer, <1 → farther.
    pub fn zoom_by(&mut self, factor: f32) {
        if factor <= 0.0 {
            return;
        }
        self.zoom = (self.zoom / factor).clamp(0.3, 500.0);
    }

    /// Frame a centered-at-origin bounding box of size `width × height` (mm).
    /// Picks a camera distance large enough that a 60°-FOV perspective view
    /// from the default 55° tilt shows the full board with ~25% margin.
    pub fn fit_to_bbox(&mut self, width: f32, height: f32) {
        // Radius of the bbox on the XY plane. The projection squashes Y by
        // cos(tilt) in screen space, so the taller dimension bounds the fit.
        let radius = (width.max(height) * 0.5).max(1.0);
        // For FOV 60° → half-FOV 30° → distance = radius / tan(30°) ≈ 1.73r.
        // Bump by 25% for visual breathing room.
        let dist = radius / 30f32.to_radians().tan() * 1.25;
        self.zoom = dist.clamp(0.3, 500.0);
    }

    /// Reset to the default tilted top-down orientation (the same view the
    /// panel opens in) with the orbit pivot back at world origin. Leaves
    /// zoom alone so the caller can follow up with `fit_to_bbox` to also
    /// re-frame the board if they want.
    pub fn reset_top_down(&mut self) {
        // True top-down — look straight down the world +Z axis. The
        // viewer's screen-space convention is +Y up (OpenGL); csgrs
        // produces meshes with +Z up (OpenSCAD / CAD convention). So
        // identity rotation lays csgrs's +Z down horizontally on
        // screen — wrong. A -90° rotation about world X maps world
        // +Z → screen +Y and world +Y → screen -Z (into the page),
        // which is the actual "plan view": XY ground plane face-on,
        // Z extruding toward the camera. Circles in the XY plane
        // render as true circles, standoffs as overhead disks.
        let tilt = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), -90f32.to_radians());
        self.rotation = tilt;
        self.target = Vector3::zeros();
    }

    /// Flip the view 180° about the world Y axis — in practice, this
    /// spins the board over so the bottom faces the camera. Pressing
    /// `F` twice returns to the original view.
    pub fn flip_y(&mut self) {
        let flip = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), std::f32::consts::PI);
        self.rotation *= flip;
    }

    /// Rotate the view about the world Z axis by `radians`. Bound to the
    /// `R` hotkey as 90° increments — each press steps the board a
    /// quarter-turn in-plane.
    pub fn rotate_in_plane(&mut self, radians: f32) {
        let rot = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), radians);
        self.rotation *= rot;
    }
}

/// Project a world-space point through `mvp` into the egui screen rect.
/// Returns `None` if the point is behind the camera. Used by the HUD layer
/// to paint axis labels at the tip of each axis line.
pub fn project(mvp: &Matrix4<f32>, viewport: egui::Rect, world: Vector3<f32>) -> Option<egui::Pos2> {
    let clip = mvp * Vector4::new(world.x, world.y, world.z, 1.0);
    if clip.w <= 0.0 {
        return None;
    }
    let ndc_x = clip.x / clip.w;
    let ndc_y = clip.y / clip.w;
    // NDC y is up, egui y is down — flip.
    let sx = viewport.center().x + ndc_x * viewport.width() * 0.5;
    let sy = viewport.center().y - ndc_y * viewport.height() * 0.5;
    Some(egui::Pos2::new(sx, sy))
}

/// Un-project a screen pixel onto the Z=0 world plane. Shoots a ray from
/// the near clip plane to the far clip plane through the pixel and
/// intersects it with the board plane — works for any camera orientation,
/// including heavily tilted perspective views where a naive inverse-of-NDC
/// trick wouldn't. Used by right-mouse-drag zoom-to-region so the selected
/// rectangle maps back to the physical region the user framed.
pub fn unproject_to_z0(
    mvp_inverse: &Matrix4<f32>,
    viewport: egui::Rect,
    screen: egui::Pos2,
) -> Option<Vector3<f32>> {
    let nx = 2.0 * (screen.x - viewport.min.x) / viewport.width() - 1.0;
    let ny = 1.0 - 2.0 * (screen.y - viewport.min.y) / viewport.height();
    let to_world = |z: f32| -> Option<Vector3<f32>> {
        let clip = Vector4::new(nx, ny, z, 1.0);
        let w = mvp_inverse * clip;
        if w.w.abs() < 1e-6 {
            return None;
        }
        Some(Vector3::new(w.x / w.w, w.y / w.w, w.z / w.w))
    };
    let near = to_world(-1.0)?;
    let far = to_world(1.0)?;
    let dz = far.z - near.z;
    if dz.abs() < 1e-6 {
        return None;
    }
    let t = -near.z / dz;
    Some(near + (far - near) * t)
}
