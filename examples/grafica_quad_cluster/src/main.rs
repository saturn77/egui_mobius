//! Renders the QuadCluster block diagram inside a [`CanvasCitizen`].
//!
//! Drag to pan, scroll to zoom, double-click to fit. Use the in-canvas ribbon
//! to toggle the grid, change grid spacing, or switch routing styles.
//!
//! Run: `cargo run -p grafica_quad_cluster`

use eframe::egui;
use egui_grafica::model::{
    Border, CanvasSettings, Edge, EdgeEnd, EdgeId, EdgeOverlay, Fill, LineStyle, Node, NodeId,
    NodeKind, Overlay, Port, PortAnchor, PortId, PortKind, Routing, Scene, TextAnchor, TextLabel,
    Transform,
};
use egui_grafica::CanvasCitizen;

// ───────────────────────────────────────────────────────────────────────────
// Scene construction — the QuadCluster sketch as Rust
// ───────────────────────────────────────────────────────────────────────────

fn build_quad_cluster_scene() -> Scene {
    let mut scene = Scene {
        name: "QuadCluster".to_string(),
        settings: CanvasSettings {
            grid_spacing: 20.0,
            snap_to_grid: true,
            show_grid: true,
            default_routing: Routing::Orthogonal,
            ..Default::default()
        },
        ..Default::default()
    };

    scene.nodes.push(power_board());
    scene.nodes.push(sense_board());
    scene.nodes.push(adc_board());
    scene.nodes.push(fpga_board());

    let blue = "#2196F3";
    let green = "#10B981";
    let red = "#EF4444";
    let grey = "#6B7280";

    scene.edges.push(orth_edge("e_p_s_1", ("power_board", "ch1"), ("sense_board", "ch1"), blue));
    scene.edges.push(orth_edge("e_p_s_2", ("power_board", "ch2"), ("sense_board", "ch2"), blue));
    scene.edges.push(orth_edge("e_p_s_3", ("power_board", "ch3"), ("sense_board", "ch3"), blue));
    scene.edges.push(orth_edge("e_p_s_4", ("power_board", "ch4"), ("sense_board", "ch4"), blue));

    scene.edges.push(orth_edge("e_s_a", ("sense_board", "adc_bus"), ("adc", "sense_bus"), green));
    scene.edges.push(orth_edge("e_a_f", ("adc", "fpga_link"), ("fpga", "adc_link"), red));

    // FPGA → power return path — dashed grey, routed *below* every
    // board with an explicit U so the wire isn't pinned to their
    // bottoms. Auto-orthogonal of two south-facing ports otherwise
    // hugs the node edges. A direction-aware router would obviate
    // these waypoints; for now the demo prescribes them.
    //
    // FPGA south(0.5) = (890, 500); power_board south(0.3) = (156, 420).
    // Drop to y = 560, well clear of the deepest board (FPGA, y = 500).
    scene.edges.push(Edge {
        id: EdgeId("e_f_p".to_string()),
        from: EdgeEnd::Port(NodeId("fpga".to_string()), PortId("fpga_intf".to_string())),
        to: EdgeEnd::Port(NodeId("power_board".to_string()), PortId("fpga_intf".to_string())),
        routing: Routing::Manual { waypoints: vec![(890.0, 560.0), (156.0, 560.0)] },
        overlay: EdgeOverlay {
            color: grey.to_string(),
            width: 1.5,
            line_style: LineStyle::Dashed,
            ..Default::default()
        },
    });

    scene
}

fn power_board() -> Node {
    Node {
        id: NodeId("power_board".to_string()),
        kind: NodeKind::Rect,
        transform: Transform { position: (120.0, 200.0), size: (120.0, 220.0), rotation: 0.0 },
        overlay: overlay_filled("#DBEAFE", "BUCK\nGaN POWER\nBOARD", TextAnchor::Center),
        ports: vec![
            port("vin_48v", PortKind::In, PortAnchor::North(0.5)),
            port("ch1", PortKind::Out, PortAnchor::East(0.2)),
            port("ch2", PortKind::Out, PortAnchor::East(0.4)),
            port("ch3", PortKind::Out, PortAnchor::East(0.6)),
            port("ch4", PortKind::Out, PortAnchor::East(0.8)),
            port("fpga_intf", PortKind::Bidir, PortAnchor::South(0.3)),
        ],
    }
}

fn sense_board() -> Node {
    Node {
        id: NodeId("sense_board".to_string()),
        kind: NodeKind::Rect,
        transform: Transform { position: (360.0, 200.0), size: (140.0, 220.0), rotation: 0.0 },
        overlay: overlay_filled("#FEF3C7", "QUAD CLUSTER\n5VDC\nSENSE BOARD", TextAnchor::TopCenter),
        ports: vec![
            port("ch1", PortKind::In, PortAnchor::West(0.2)),
            port("ch2", PortKind::In, PortAnchor::West(0.4)),
            port("ch3", PortKind::In, PortAnchor::West(0.6)),
            port("ch4", PortKind::In, PortAnchor::West(0.8)),
            port("adc_bus", PortKind::Out, PortAnchor::East(0.5)),
        ],
    }
}

fn adc_board() -> Node {
    Node {
        id: NodeId("adc".to_string()),
        kind: NodeKind::Rect,
        transform: Transform { position: (580.0, 320.0), size: (120.0, 90.0), rotation: 0.0 },
        overlay: overlay_filled("#FCA5A5", "ADC BOARD\nDAUGHTER", TextAnchor::Center),
        ports: vec![
            port("sense_bus", PortKind::In, PortAnchor::West(0.5)),
            port("fpga_link", PortKind::Bidir, PortAnchor::East(0.5)),
        ],
    }
}

fn fpga_board() -> Node {
    Node {
        id: NodeId("fpga".to_string()),
        kind: NodeKind::Rect,
        transform: Transform { position: (800.0, 260.0), size: (180.0, 240.0), rotation: 0.0 },
        overlay: overlay_filled("#A7F3D0", "FPGA BOARD", TextAnchor::Center),
        ports: vec![
            port("adc_link", PortKind::Bidir, PortAnchor::West(0.25)),
            port("fpga_intf", PortKind::In, PortAnchor::South(0.5)),
        ],
    }
}

fn overlay_filled(fill_hex: &str, label: &str, anchor: TextAnchor) -> Overlay {
    Overlay {
        border: Border { color: "#1F2937".to_string(), width: 2.0, style: LineStyle::Solid },
        fill: Fill { color: fill_hex.to_string(), alpha: 0.90 },
        text: Some(TextLabel {
            value: label.to_string(),
            anchor,
            font_family: String::new(),
            font_size: 12.0,
            bold: false,
            italic: false,
            color: "#111827".to_string(),
        }),
    }
}

fn port(name: &str, kind: PortKind, anchor: PortAnchor) -> Port {
    Port {
        id: PortId(name.to_string()),
        name: name.to_string(),
        kind,
        anchor,
        data_type: None,
    }
}

fn orth_edge(id: &str, from: (&str, &str), to: (&str, &str), color: &str) -> Edge {
    Edge {
        id: EdgeId(id.to_string()),
        from: EdgeEnd::Port(NodeId(from.0.to_string()), PortId(from.1.to_string())),
        to: EdgeEnd::Port(NodeId(to.0.to_string()), PortId(to.1.to_string())),
        routing: Routing::Orthogonal,
        overlay: EdgeOverlay {
            color: color.to_string(),
            width: 2.0,
            line_style: LineStyle::Solid,
            ..Default::default()
        },
    }
}

// ───────────────────────────────────────────────────────────────────────────
// eframe App
// ───────────────────────────────────────────────────────────────────────────

struct DemoApp {
    citizen: CanvasCitizen,
    /// Set on first frame so the initial view fits the scene to the window.
    initial_fit_done: bool,
}

impl DemoApp {
    fn new() -> Self {
        Self {
            citizen: CanvasCitizen::new(build_quad_cluster_scene()),
            initial_fit_done: false,
        }
    }
}

impl eframe::App for DemoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            if !self.initial_fit_done {
                // Approximate the canvas rect on first frame — ribbon hasn't
                // rendered yet so we use the whole panel as a hint.
                let rect = ui.available_rect_before_wrap();
                self.citizen.viewport = egui_grafica::citizen::fit_viewport_to_scene(
                    &self.citizen.registry.scene(),
                    rect,
                    self.citizen.fit_padding,
                );
                self.initial_fit_done = true;
            }
            self.citizen.show(ui);
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "egui_grafica — QuadCluster demo",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 700.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            egui_grafica::install_fonts(&cc.egui_ctx);
            // Register the wgpu canvas pipeline. Present only on the wgpu
            // backend — eframe's default renderer here.
            if let Some(render_state) = cc.wgpu_render_state.as_ref() {
                egui_grafica::gpu::init(render_state);
            }
            Ok(Box::new(DemoApp::new()))
        }),
    )
}
