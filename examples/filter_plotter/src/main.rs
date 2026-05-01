//! filter_plotter — a citizen-pattern tutorial app.
//!
//! Three panels (Plot / Settings / Terminal) wired into `egui_dock` via a
//! `TabViewer`, with `egui_citizen::Dispatcher` as the message hub
//! between the settings panel and an in-process IIR filter backend.
//!
//! Click Generate in the Settings panel → AppMessage::Generate flows
//! through the settings outbox → drain loop calls the backend → traces
//! land in SharedState → plot panel renders them on the next frame.

mod backend;
mod dispatcher;
mod messages;
mod panels;
mod state;
mod tabs;
mod theme;

use eframe::egui;
use egui_citizen::Dispatcher;
use egui_dock::{DockArea, DockState, NodeIndex};

use backend::iir::InProcessIir;
use panels::{logger::LoggerPanel, plot::PlotPanel, settings::SettingsPanel};
use state::SharedState;
use tabs::{Tab, TabKind, TabViewer};

struct App {
    dispatcher: Dispatcher,
    dock_state: DockState<Tab>,
    state: SharedState,
    plot: PlotPanel,
    settings: SettingsPanel,
    logger: LoggerPanel,
    backend: InProcessIir,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_visuals(&cc.egui_ctx);

        let mut dispatcher_handle = Dispatcher::new();
        let citizens = dispatcher::register_citizens(&mut dispatcher_handle);

        // Dock layout:
        //   ┌──────────────┬─────────────┐
        //   │              │  Settings   │
        //   │     Plot     ├─────────────┤
        //   │              │  Logger     │
        //   └──────────────┴─────────────┘
        let mut dock_state = DockState::new(vec![Tab::new(TabKind::Plot)]);
        let [_, right] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.65,
            vec![Tab::new(TabKind::Settings)],
        );
        let [_, _bottom] =
            dock_state
                .main_surface_mut()
                .split_below(right, 0.55, vec![Tab::new(TabKind::Logger)]);

        let state = SharedState::new();
        dispatcher::append_log(&state.log, "[INFO] filter_plotter started".into());

        Self {
            dispatcher: dispatcher_handle,
            dock_state,
            state,
            plot: PlotPanel::new(citizens.plot),
            settings: SettingsPanel::new(citizens.settings),
            logger: LoggerPanel::new(citizens.logger),
            backend: InProcessIir::new(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        DockArea::new(&mut self.dock_state).show_inside(
            ui,
            &mut TabViewer {
                state: &self.state,
                dispatcher: &mut self.dispatcher,
                plot: &mut self.plot,
                settings: &mut self.settings,
                logger: &mut self.logger,
            },
        );

        // Drain pass — once per frame, after the dock has rendered and
        // any on_tab_button or in-panel events have queued.
        dispatcher::drain_citizen(&mut self.dispatcher, &self.state.log);

        let outbox = std::mem::take(&mut self.settings.outbox);
        for msg in outbox {
            dispatcher::handle(msg, &self.state, &mut self.backend, &self.state.log);
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "filter_plotter",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 750.0])
                .with_min_inner_size([800.0, 500.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
