//! Getting Started — complete working example from docs/getting-started.md
//!
//! Three panels: Config, Display, Logger.
//! Click a tab header → Dispatcher activates that citizen.
//! Logger shows all lifecycle messages flowing through.
//!
//! Run: cargo run -p getting_started

use eframe::egui;
use egui::Color32;
use egui_citizen::{Citizen, CitizenId, CitizenMessage, CitizenState, Dispatcher};
use egui_dock::{DockArea, DockState, NodeIndex};

// ── Panel structs implementing Citizen ──────────────────────────────────

struct ConfigPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    value: f32,
}

impl ConfigPanel {
    fn new(state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new("config"),
            citizen_state: state,
            value: 50.0,
        }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configuration");
        ui.add_space(8.0);
        ui.add(egui::Slider::new(&mut self.value, 0.0..=100.0).text("Value"));
        ui.add_space(8.0);
        if self.is_active() {
            ui.label(
                egui::RichText::new("This panel is active")
                    .color(Color32::from_rgb(0x9e, 0xce, 0x6a)),
            );
        } else {
            ui.label(
                egui::RichText::new("Click this tab to activate")
                    .color(Color32::from_rgb(0x56, 0x5f, 0x89)),
            );
        }
    }
}

impl Citizen for ConfigPanel {
    fn id(&self) -> &CitizenId {
        &self.citizen_id
    }
    fn citizen_state(&self) -> &CitizenState {
        &self.citizen_state
    }
    fn citizen_state_mut(&mut self) -> &mut CitizenState {
        &mut self.citizen_state
    }
}

struct DisplayPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
}

impl DisplayPanel {
    fn new(state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new("display"),
            citizen_state: state,
        }
    }

    fn show(&self, ui: &mut egui::Ui) {
        ui.heading("Display");
        ui.add_space(8.0);
        if self.is_active() {
            ui.label(
                egui::RichText::new("This panel is active")
                    .color(Color32::from_rgb(0x9e, 0xce, 0x6a)),
            );
        } else {
            ui.label(
                egui::RichText::new("Click this tab to activate")
                    .color(Color32::from_rgb(0x56, 0x5f, 0x89)),
            );
        }
    }
}

impl Citizen for DisplayPanel {
    fn id(&self) -> &CitizenId {
        &self.citizen_id
    }
    fn citizen_state(&self) -> &CitizenState {
        &self.citizen_state
    }
    fn citizen_state_mut(&mut self) -> &mut CitizenState {
        &mut self.citizen_state
    }
}

// ── Tabs ────────────────────────────────────────────────────────────────

#[derive(Clone)]
enum TabKind {
    Config,
    Display,
    Logger,
}

#[derive(Clone)]
struct Tab {
    kind: TabKind,
}

impl Tab {
    fn title(&self) -> &str {
        match self.kind {
            TabKind::Config => "Config",
            TabKind::Display => "Display",
            TabKind::Logger => "Logger",
        }
    }

    fn citizen_id(&self) -> Option<CitizenId> {
        match self.kind {
            TabKind::Config => Some(CitizenId::new("config")),
            TabKind::Display => Some(CitizenId::new("display")),
            TabKind::Logger => None, // logger is passive, not a citizen
        }
    }
}

// ── TabViewer bridges egui_dock to the Dispatcher ───────────────────────

struct TabViewer<'a> {
    dispatcher: &'a mut Dispatcher,
    config: &'a mut ConfigPanel,
    display: &'a DisplayPanel,
    log: &'a mut Vec<String>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
        if response.clicked()
            && let Some(id) = tab.citizen_id()
        {
            self.dispatcher.activate(&id);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        match tab.kind {
            TabKind::Config => self.config.show(ui),
            TabKind::Display => self.display.show(ui),
            TabKind::Logger => {
                ui.heading("Messages");
                ui.add_space(4.0);
                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in self.log.iter() {
                            let color = if line.contains("Activated") {
                                Color32::from_rgb(0x9e, 0xce, 0x6a)
                            } else {
                                Color32::from_rgb(0x56, 0x5f, 0x89)
                            };
                            ui.label(egui::RichText::new(line).color(color).monospace());
                        }
                    });
            }
        }
    }
}

// ── App ─────────────────────────────────────────────────────────────────

struct MyApp {
    dock_state: DockState<Tab>,
    dispatcher: Dispatcher,
    config: ConfigPanel,
    display: DisplayPanel,
    log: Vec<String>,
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Step 1: Create Dispatcher and register citizens
        let mut dispatcher = Dispatcher::new();
        let config_state = dispatcher.register(CitizenId::new("config"));
        let display_state = dispatcher.register(CitizenId::new("display"));

        // Activate config by default
        dispatcher.activate(&CitizenId::new("config"));
        let _ = dispatcher.drain_messages();

        // Create panel structs with their shared state handles
        let config = ConfigPanel::new(config_state);
        let display = DisplayPanel::new(display_state);

        // Dock layout
        let mut dock_state = DockState::new(vec![Tab {
            kind: TabKind::Display,
        }]);
        let [left, _right] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.35,
            vec![Tab {
                kind: TabKind::Config,
            }],
        );
        dock_state.main_surface_mut().split_below(
            left,
            0.65,
            vec![Tab {
                kind: TabKind::Logger,
            }],
        );

        Self {
            dock_state,
            dispatcher,
            config,
            display,
            log: vec!["App started".to_string()],
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Step 2 + 3: Render dock, then drain messages
        let mut dock_state = self.dock_state.clone();
        let mut dispatcher = std::mem::take(&mut self.dispatcher);
        {
            let mut viewer = TabViewer {
                dispatcher: &mut dispatcher,
                config: &mut self.config,
                display: &self.display,
                log: &mut self.log,
            };
            DockArea::new(&mut dock_state).show_inside(ui, &mut viewer);
        }

        // Drain citizen lifecycle messages
        for msg in dispatcher.drain_messages() {
            match &msg {
                CitizenMessage::Activated { id } => {
                    self.log.push(format!("[CITIZEN] Activated: {}", id));
                }
                CitizenMessage::Deactivated { id } => {
                    self.log.push(format!("[CITIZEN] Deactivated: {}", id));
                }
                _ => {}
            }
        }

        self.dispatcher = dispatcher;
        self.dock_state = dock_state;
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Getting Started — egui-citizen",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([700.0, 450.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
