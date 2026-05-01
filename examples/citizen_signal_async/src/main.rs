//! citizen_signal_async — citizen pattern + egui_mobius signal/slot + Tokio backend.
//!
//! Phase 3 — three docked citizens (Control / Result / Logger) wired
//! through `egui_dock`'s `TabViewer`. Tab clicks forward to
//! `Dispatcher::activate` so citizen lifecycle messages flow through
//! the drain loop and end up in the Logger panel.
//!
//! Click Compute on the Control panel → AppMessage::Compute hits the
//! outbox → drain loop forwards via `work_signal.send(req)` → backend
//! runs on Tokio → `result_signal` carries the response back → result
//! slot writes `last_result` and `in_flight` on `SharedState` and
//! requests a UI repaint.

mod backend;
mod dispatcher;
mod messages;
mod panels;
mod state;
mod tabs;

use eframe::egui;
use egui_citizen::Dispatcher as CitizenDispatcher;
use egui_dock::{DockArea, DockState, NodeIndex};
use egui_mobius::signals::Signal;

use crate::panels::{control::ControlPanel, logger::LoggerPanel, result::ResultPanel};
use crate::state::{SharedState, WorkRequest};
use crate::tabs::{Tab, TabKind, TabViewer};

struct App {
    dispatcher: CitizenDispatcher,
    dock_state: DockState<Tab>,
    state: SharedState,
    work_signal: Signal<WorkRequest>,
    /// Keeps the Tokio runtime alive. Dropping this silences the backend.
    _backend: backend::BackendHandle,
    control: ControlPanel,
    result:  ResultPanel,
    logger:  LoggerPanel,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = SharedState::new();

        let mut dispatcher = CitizenDispatcher::new();
        let citizens = dispatcher::register_citizens(&mut dispatcher);

        // Dock layout:
        //   ┌──────────────┬─────────────┐
        //   │              │   Control   │
        //   │    Result    ├─────────────┤
        //   │              │   Logger    │
        //   └──────────────┴─────────────┘
        let mut dock_state = DockState::new(vec![Tab::new(TabKind::Result)]);
        let [_, right] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.6,
            vec![Tab::new(TabKind::Control)],
        );
        let [_, _bottom] = dock_state.main_surface_mut().split_below(
            right,
            0.5,
            vec![Tab::new(TabKind::Logger)],
        );

        let (work_signal, mut result_slot, backend_handle) = backend::wire_backend();

        // Result-slot handler — runs on the slot's worker thread when a
        // backend response arrives. Writes through the shared `Dynamic`
        // handles, then wakes the UI so the new value paints next frame.
        let last_result = state.last_result.clone();
        let in_flight   = state.in_flight.clone();
        let log         = state.log.clone();
        let ctx         = cc.egui_ctx.clone();
        result_slot.start(move |resp| {
            last_result.set(resp.value);
            in_flight.set(false);
            dispatcher::append_log(
                &log,
                format!(
                    "[backend] result: value={:.4} elapsed_ms={}",
                    resp.value, resp.elapsed_ms,
                ),
            );
            ctx.request_repaint();
        });

        dispatcher::append_log(&state.log, "[INFO] citizen_signal_async started".into());

        Self {
            dispatcher,
            dock_state,
            state,
            work_signal,
            _backend: backend_handle,
            control: ControlPanel::new(citizens.control),
            result:  ResultPanel::new(citizens.result),
            logger:  LoggerPanel::new(citizens.logger),
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
                control: &mut self.control,
                result:  &mut self.result,
                logger:  &mut self.logger,
            },
        );

        // Drain pass — once per frame, after the dock has rendered.
        dispatcher::drain_citizen(&mut self.dispatcher, &self.state.log);

        let outbox = std::mem::take(&mut self.control.outbox);
        for msg in outbox {
            dispatcher::handle(msg, &self.state, &self.work_signal, &self.state.log);
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "citizen_signal_async",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1000.0, 650.0])
                .with_min_inner_size([700.0, 450.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
