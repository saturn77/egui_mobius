//! Viewer3D + csgrs demo — renders a 6"×4" PCB mounting plate with
//! six mounting holes built via constructive solid geometry.
//!
//! The whole point is to prove the assembly path end-to-end:
//!
//! 1. csgrs builds the plate as a cuboid minus six cylinder cuts.
//! 2. csgrs's mesh gets tessellated and converted to the viewer's
//!    `xyz rgb` flat-buffer format.
//! 3. `ViewerCitizen::set_scene_triangles` hands it to the viewer.
//! 4. The viewer renders with all the creature comforts: orbit,
//!    wheel zoom, right-drag zoom-to-region, double-click reset,
//!    M for measure, G for grid.
//!
//! Run: `cargo run -p viewer3d_csgrs`

use eframe::egui;
use egui_3d_viewer::ViewerCitizen;
use egui_citizen::{CitizenId, Dispatcher};

type Mesh = csgrs::mesh::Mesh<()>;

// ── Plate dimensions in mm ──────────────────────────────────────────────

/// Plate width (X) — 6 inches in mm.
const PLATE_W: f64 = 152.4;
/// Plate length (Y) — 4 inches in mm.
const PLATE_L: f64 = 101.6;
/// Plate thickness (Z) — 1/8 inch in mm.
const PLATE_T: f64 = 3.175;

/// M3-clearance hole radius in mm.
const HOLE_R: f64 = 1.75;
/// Edge-to-hole offset, mm.
const HOLE_INSET: f64 = 5.0;

/// Aluminium-grey for the plate triangles.
const PLATE_COLOR: [f32; 3] = [0.65, 0.68, 0.72];

// ── CSG construction ────────────────────────────────────────────────────

/// Build the mounting plate: a cuboid minus six cylinder cuts. Returns a
/// triangulated mesh re-centred on world origin.
fn build_plate() -> Mesh {
    use csgrs::traits::CSG;

    let plate = Mesh::cuboid(PLATE_W, PLATE_L, PLATE_T, None);

    // Hole positions in plate-local (corner-anchored) coordinates.
    // Four corners + two mid-x at the long edges.
    let mid_x = PLATE_W / 2.0;
    let positions = [
        (HOLE_INSET, HOLE_INSET),
        (PLATE_W - HOLE_INSET, HOLE_INSET),
        (HOLE_INSET, PLATE_L - HOLE_INSET),
        (PLATE_W - HOLE_INSET, PLATE_L - HOLE_INSET),
        (mid_x, HOLE_INSET),
        (mid_x, PLATE_L - HOLE_INSET),
    ];

    // Subtract each hole. Cylinders extend slightly past the plate on
    // both sides so the through-cut is clean — no thin "skin" left from
    // numerical jitter.
    let mut result = plate;
    for (x, y) in positions {
        let hole = Mesh::cylinder(HOLE_R, PLATE_T + 1.0, 24, None)
            .translate(x, y, -0.5);
        result = result.difference(&hole);
    }

    // Re-centre on world origin so the camera defaults frame the plate.
    result.translate(-PLATE_W / 2.0, -PLATE_L / 2.0, 0.0)
}

/// Convert a csgrs mesh to a flat `xyz rgb` triangle buffer in the
/// shape `ColoredMesh::upload` expects — six floats per vertex,
/// three vertices per triangle.
fn mesh_to_xyz_rgb(mesh: &Mesh, color: [f32; 3]) -> Vec<f32> {
    let triangulated = mesh.triangulate();
    let [r, g, b] = color;
    let mut out = Vec::with_capacity(triangulated.polygons.len() * 18);
    for poly in &triangulated.polygons {
        // After triangulate(), each polygon has exactly 3 vertices.
        for v in &poly.vertices {
            out.push(v.pos.x as f32);
            out.push(v.pos.y as f32);
            out.push(v.pos.z as f32);
            out.push(r);
            out.push(g);
            out.push(b);
        }
    }
    out
}

// ── App ─────────────────────────────────────────────────────────────────

struct ViewerApp {
    viewer: ViewerCitizen,
    /// Holds the dispatcher so its CitizenState handles stay alive,
    /// even though this demo doesn't activate or drain it.
    _dispatcher: Dispatcher,
}

impl ViewerApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut dispatcher = Dispatcher::new();
        let viewer_state = dispatcher.register(CitizenId::new("viewer"));
        let mut viewer = ViewerCitizen::new("viewer", viewer_state);

        // Build the plate, hand it to the viewer, frame the camera.
        let plate = build_plate();
        let verts = mesh_to_xyz_rgb(&plate, PLATE_COLOR);
        viewer.set_scene_triangles(verts);

        // Scale the axes gizmo to ~15 % of the plate's largest dimension
        // and frame the camera so the plate fills the viewport.
        let max_dim = PLATE_W.max(PLATE_L) as f32;
        viewer.set_axes_length((max_dim * 0.15).max(3.0));
        viewer.camera_mut().fit_to_bbox(PLATE_W as f32, PLATE_L as f32);

        Self {
            viewer,
            _dispatcher: dispatcher,
        }
    }
}

impl eframe::App for ViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let gl = frame.gl();
        self.viewer.show(ui, gl);
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "viewer3d_csgrs — egui_3d_viewer + csgrs mounting plate demo",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1024.0, 720.0])
                .with_title("3D Viewer Citizen — csgrs Mounting Plate"),
            renderer: eframe::Renderer::Glow,
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(ViewerApp::new(cc)))),
    )
}
