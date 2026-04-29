//! citizen_signal_async — citizen pattern + egui_mobius signal/slot + Tokio backend.
//!
//! Phase 2 — eframe shell + backend wiring + smoke-test central panel.
//! The three-citizen dock layout (Control / Result / Logger) follows in
//! Phase 3; for now everything renders into a single `CentralPanel` so
//! the cross-thread bus can be exercised end to end.

mod backend;
mod dispatcher;
mod messages;
mod state;

use eframe::egui;
use egui_citizen::Dispatcher as CitizenDispatcher;
use egui_mobius::signals::Signal;

use crate::messages::AppMessage;
use crate::state::{SharedState, WorkRequest};

struct App {
    dispatcher: CitizenDispatcher,
    state: SharedState,
    work_signal: Signal<WorkRequest>,
    /// Keeps the Tokio runtime alive. Dropping this silences the backend.
    _backend: backend::BackendHandle,
    /// Outbox of AppMessages produced this frame; drained at end of `update()`.
    outbox: Vec<AppMessage>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let state = SharedState::new();

        // Citizen dispatcher — registered citizens exist now even though
        // panels won't be wired to them until Phase 3. The handles
        // (`_citizens`) are intentionally discarded for now.
        let mut dispatcher = CitizenDispatcher::new();
        let _citizens = dispatcher::register_citizens(&mut dispatcher);

        // Build the cross-thread bus.
        let (work_signal, mut result_slot, backend_handle) = backend::wire_backend();

        // Result slot handler — runs on the slot's worker thread when a
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
            state,
            work_signal,
            _backend: backend_handle,
            outbox: Vec::new(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("citizen_signal_async — smoke test");
            ui.add_space(8.0);

            // Sliders bind to the reactive params via the standard
            // get / set pattern.
            let mut dur = self.state.params.work_duration_ms.get();
            if ui
                .add(egui::Slider::new(&mut dur, 50..=3000).text("duration_ms"))
                .changed()
            {
                self.state.params.work_duration_ms.set(dur);
            }

            let mut seed = self.state.params.seed.get();
            if ui
                .add(egui::Slider::new(&mut seed, 0.0..=1.0).text("seed"))
                .changed()
            {
                self.state.params.seed.set(seed);
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("Compute").clicked() {
                    self.outbox.push(AppMessage::Compute);
                }
                if self.state.in_flight.get() {
                    ui.spinner();
                    ui.label("in flight…");
                }
            });

            ui.add_space(8.0);
            ui.label(format!("last result: {:.6}", self.state.last_result.get()));

            ui.add_space(12.0);
            ui.separator();
            ui.label("Log:");
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in self.state.log.get().iter() {
                        ui.monospace(line);
                    }
                });
        });

        // Drain pass — once per frame, after the UI has rendered and
        // any in-frame events have queued.
        dispatcher::drain_citizen(&mut self.dispatcher, &self.state.log);

        let outbox = std::mem::take(&mut self.outbox);
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
                .with_inner_size([900.0, 600.0])
                .with_min_inner_size([600.0, 400.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
