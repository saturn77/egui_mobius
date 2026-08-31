//! serial_monitor — the citizen pattern against real hardware.
//!
//! Three docked citizens (Monitor / Console / Logger) over a serial
//! worker thread. The Monitor pushes intents onto its outbox; the drain
//! loop forwards them through `monitor_actions::handle`, which drives
//! the worker over an mpsc channel. The worker writes received lines
//! and connection state into shared `Dynamic` cells — Path A carries
//! them to the Console and Monitor on the next frame. Serial core
//! adapted from SaturnGridSim.
//!
//! No hardware handy? Make a virtual pair and echo into one end:
//!
//! ```text
//! socat -d -d pty,raw,echo=0 pty,raw,echo=0
//! # note the two /dev/pts/N it prints; connect to one, then:
//! while true; do echo "tick $(date +%T)" > /dev/pts/M; sleep 1; done
//! ```

mod backend;
mod citizens;
mod messages;
mod monitor_actions;
mod state;
mod tabs;
mod theme;

use eframe::egui;
use egui_citizen::Registry;
use egui_dock::{DockArea, DockState, NodeIndex};

use crate::citizens::{console::ConsolePanel, logger::LoggerPanel, monitor::MonitorPanel};
use crate::messages::AppMessage;
use crate::state::SharedState;
use crate::tabs::{Tab, TabKind, TabViewer};

struct App {
    registry: Registry,
    dock_state: DockState<Tab>,
    state: SharedState,
    backend: backend::SerialBackend,
    monitor: MonitorPanel,
    console: ConsolePanel,
    logger: LoggerPanel,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply(&cc.egui_ctx);

        let state = SharedState::new();
        let mut backend = backend::SerialBackend::new(cc.egui_ctx.clone());

        // Register every citizen up front; panels hold the CitizenState
        // handles the registry gives back (never CitizenState::new()).
        let mut registry = Registry::new();
        let monitor_state = registry.add().with_name(tabs::MONITOR_ID);
        let console_state = registry.add().with_name(tabs::CONSOLE_ID);
        let logger_state  = registry.add().with_name(tabs::LOGGER_ID);
        registry.activate(tabs::MONITOR_ID);

        // Dock layout:
        //   ┌──────────────┬─────────────┐
        //   │              │   Monitor   │
        //   │   Console    ├─────────────┤
        //   │              │   Logger    │
        //   └──────────────┴─────────────┘
        let mut dock_state = DockState::new(vec![Tab::new(TabKind::Console)]);
        let [_, right] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.6,
            vec![Tab::new(TabKind::Monitor)],
        );
        let [_, _bottom] =
            dock_state
                .main_surface_mut()
                .split_below(right, 0.55, vec![Tab::new(TabKind::Logger)]);

        // Enumerate ports once at startup so the combo isn't empty.
        monitor_actions::handle(AppMessage::RefreshPorts, &state, &mut backend);
        citizens::append_log(&state.log, "[INFO] serial_monitor started".into());

        Self {
            registry,
            dock_state,
            state,
            backend,
            monitor: MonitorPanel::new(monitor_state),
            console: ConsolePanel::new(console_state),
            logger: LoggerPanel::new(logger_state),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.dock_state).show_inside(
            ui,
            &mut TabViewer {
                state: &self.state,
                registry: &mut self.registry,
                monitor: &mut self.monitor,
                console: &mut self.console,
                logger: &mut self.logger,
            },
        );

        // Drain pass — once per frame, after the dock has rendered.
        citizens::drain_citizen(&mut self.registry, &self.state.log);

        // Both emitting citizens drain through the same actions funnel —
        // the Monitor's connection intents and the Console's TX lines.
        let outbox = std::mem::take(&mut self.monitor.outbox);
        for msg in outbox {
            monitor_actions::handle(msg, &self.state, &mut self.backend);
        }
        let outbox = std::mem::take(&mut self.console.outbox);
        for msg in outbox {
            monitor_actions::handle(msg, &self.state, &mut self.backend);
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "serial_monitor",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1000.0, 620.0])
                .with_min_inner_size([700.0, 450.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}




