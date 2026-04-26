//! Citizen Dock — demonstrates `egui_citizen` with `egui_dock`.
//!
//! This is the basic example showing citizen lifecycle management.
//!
//! Three algorithm tabs (Alpha, Beta, Gamma) each act as citizens.
//! Clicking a tab header activates that citizen and the plot panel
//! reactively shows the corresponding curve — no per-frame fighting.
//!
//! Pattern taken from quarri / saturn-grid-sim dock architecture.

use eframe::egui;
use egui::Color32;
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_citizen::{CitizenMessage, Dispatcher};
use egui_citizen::message::CitizenId;
use egui_mobius_reactive::Dynamic;

// ---------------------------------------------------------------------------
// Citizen IDs
// ---------------------------------------------------------------------------

const ALPHA: &str = "alpha";
const BETA: &str = "beta";
const GAMMA: &str = "gamma";

// ---------------------------------------------------------------------------
// Tabs
// ---------------------------------------------------------------------------

enum TabKind {
    Alpha,
    Beta,
    Gamma,
    Plot,
    Logger,
}

struct Tab {
    kind: TabKind,
}

impl Tab {
    fn new(kind: TabKind) -> Self {
        Self { kind }
    }

    fn title(&self) -> &str {
        match self.kind {
            TabKind::Alpha  => "Alpha",
            TabKind::Beta   => "Beta",
            TabKind::Gamma  => "Gamma",
            TabKind::Plot   => "Plot",
            TabKind::Logger => "Logger",
        }
    }

    /// Map algo tab kinds to their citizen ID, if applicable.
    fn citizen_id(&self) -> Option<CitizenId> {
        match self.kind {
            TabKind::Alpha => Some(CitizenId::new(ALPHA)),
            TabKind::Beta  => Some(CitizenId::new(BETA)),
            TabKind::Gamma => Some(CitizenId::new(GAMMA)),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// Tab viewer bridge
// ---------------------------------------------------------------------------

struct TabViewer<'a> {
    dispatcher: &'a mut Dispatcher,
    active_algo: &'a Dynamic<String>,
    log: &'a mut Vec<String>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.clicked() {
            if let Some(id) = tab.citizen_id() {
                self.dispatcher.activate(&id);
                self.active_algo.set(id.0.clone());

                // Drain messages into the log so we can see the one-hot activation
                for msg in self.dispatcher.drain_messages() {
                    match &msg {
                        CitizenMessage::Activated { id } => {
                            self.log.push(format!("[CITIZEN] Activated: {id}"));
                        }
                        CitizenMessage::Deactivated { id } => {
                            self.log.push(format!("[CITIZEN] Deactivated: {id}"));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.kind {
            TabKind::Alpha => render_algo_panel(ui, "Alpha", "Frequency response curve", CYAN),
            TabKind::Beta  => render_algo_panel(ui, "Beta", "Voltage droop curve", GREEN),
            TabKind::Gamma => render_algo_panel(ui, "Gamma", "Reactive power curve", MAGENTA),
            TabKind::Plot  => render_plot_panel(ui, &self.active_algo.get()),
            TabKind::Logger => render_logger(ui, self.log),
        }
    }
}

// ---------------------------------------------------------------------------
// Panel renderers
// ---------------------------------------------------------------------------

const BG_DARK: Color32 = Color32::from_rgb(0x1a, 0x1b, 0x26);
const CYAN: Color32 = Color32::from_rgb(0x7d, 0xcf, 0xff);
const GREEN: Color32 = Color32::from_rgb(0x9e, 0xce, 0x6a);
const MAGENTA: Color32 = Color32::from_rgb(0xbb, 0x9a, 0xf7);
const COMMENT: Color32 = Color32::from_rgb(0x56, 0x5f, 0x89);

fn render_algo_panel(ui: &mut egui::Ui, name: &str, description: &str, color: Color32) {
    egui::Frame::new()
        .fill(BG_DARK)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.heading(egui::RichText::new(format!("{name} Configuration")).color(color));
            ui.add_space(8.0);
            ui.label(description);
            ui.add_space(8.0);
            ui.label(
                egui::RichText::new("Click this tab header to activate the citizen.\nThe plot panel will switch reactively.")
                    .color(COMMENT),
            );
        });
}

fn render_plot_panel(ui: &mut egui::Ui, active: &str) {
    let (label, color) = match active {
        ALPHA => ("Alpha Curve", CYAN),
        BETA  => ("Beta Curve", GREEN),
        GAMMA => ("Gamma Curve", MAGENTA),
        _     => ("No algo selected", COMMENT),
    };

    egui::Frame::new()
        .fill(BG_DARK)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.heading(egui::RichText::new(label).color(color).strong());
            ui.add_space(8.0);

            // Draw a simple placeholder curve
            let available = ui.available_size();
            let (rect, _response) = ui.allocate_exact_size(
                egui::vec2(available.x, (available.y - 30.0).max(100.0)),
                egui::Sense::hover(),
            );
            let painter = ui.painter_at(rect);

            // Background
            painter.rect_filled(rect, 4.0, Color32::from_rgb(0x16, 0x16, 0x1e));

            // Simple curve based on active algo
            let n = 100;
            let points: Vec<egui::Pos2> = (0..=n)
                .map(|i| {
                    let t = i as f32 / n as f32;
                    let x = rect.left() + t * rect.width();
                    let y = match active {
                        ALPHA => rect.center().y - 40.0 * (t * 6.0).sin(),
                        BETA  => rect.center().y - 30.0 * (1.0 - t).powf(0.5) * (t * 4.0).cos(),
                        GAMMA => rect.center().y - 50.0 * t * (1.0 - t) * (t * 8.0).sin(),
                        _     => rect.center().y,
                    };
                    egui::pos2(x, y)
                })
                .collect();

            for pair in points.windows(2) {
                painter.line_segment([pair[0], pair[1]], egui::Stroke::new(2.5, color));
            }
        });
}

fn render_logger(ui: &mut egui::Ui, log: &[String]) {
    egui::Frame::new()
        .fill(BG_DARK)
        .inner_margin(8.0)
        .show(ui, |ui| {
            ui.heading(egui::RichText::new("Citizen Messages").color(CYAN));
            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in log {
                        let color = if line.contains("Activated") {
                            GREEN
                        } else if line.contains("Deactivated") {
                            COMMENT
                        } else {
                            Color32::WHITE
                        };
                        ui.label(egui::RichText::new(line).color(color).monospace());
                    }
                });
        });
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct CitizenDockApp {
    dock_state: DockState<Tab>,
    dispatcher: Dispatcher,
    active_algo: Dynamic<String>,
    log: Vec<String>,
}

impl CitizenDockApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Register citizens
        let mut dispatcher = Dispatcher::new();
        dispatcher.register(CitizenId::new(ALPHA));
        dispatcher.register(CitizenId::new(BETA));
        dispatcher.register(CitizenId::new(GAMMA));

        // Activate Alpha by default
        let active_algo = Dynamic::new(ALPHA.to_string());
        dispatcher.activate(&CitizenId::new(ALPHA));

        // Dock layout:
        // ┌──────────────┬──────────────┐
        // │ Alpha │ Beta │              │
        // │       │Gamma │     Plot     │
        // ├──────────────┼──────────────┤
        // │           Logger            │
        // └─────────────────────────────┘
        let mut dock_state = DockState::new(vec![Tab::new(TabKind::Plot)]);

        let [config_area, _plot] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.30,
            vec![
                Tab::new(TabKind::Alpha),
                Tab::new(TabKind::Beta),
                Tab::new(TabKind::Gamma),
            ],
        );

        let [_top, _logger] = dock_state.main_surface_mut().split_below(
            config_area,
            0.70,
            vec![Tab::new(TabKind::Logger)],
        );

        let mut log = vec!["[INFO] Citizen Dock example started".to_string()];
        for msg in dispatcher.drain_messages() {
            if let CitizenMessage::Activated { id } = &msg {
                log.push(format!("[CITIZEN] Activated: {id}"));
            }
        }

        Self {
            dock_state,
            dispatcher,
            active_algo,
            log,
        }
    }
}

impl eframe::App for CitizenDockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.dock_state).show(
            ctx,
            &mut TabViewer {
                dispatcher: &mut self.dispatcher,
                active_algo: &self.active_algo,
                log: &mut self.log,
            },
        );
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Citizen Dock Example",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([900.0, 600.0])
                .with_min_inner_size([600.0, 400.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(CitizenDockApp::new(cc)))),
    )
}
