//! Control panel — duration / seed sliders and the Compute button.
//! Click pushes `AppMessage::Compute` onto the outbox; main.rs drains
//! it and forwards through `dispatcher::handle` to the work signal.

use eframe::egui;

use crate::messages::AppMessage;
use crate::state::SharedState;

pub struct ControlPanel {
    /// Outgoing app-level messages. Populated by show() and drained by
    /// main.rs each frame.
    pub outbox: Vec<AppMessage>,
}

impl ControlPanel {
    pub fn new() -> Self {
        Self { outbox: Vec::new() }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        ui.heading("Control");
        ui.add_space(8.0);

        let mut dur = state.params.work_duration_ms.get();
        if ui
            .add(egui::Slider::new(&mut dur, 50..=3000).text("duration_ms"))
            .changed()
        {
            state.params.work_duration_ms.set(dur);
        }

        let mut seed = state.params.seed.get();
        if ui
            .add(
                egui::Slider::new(&mut seed, 0.0..=1.0)
                    .text("seed")
                    .fixed_decimals(3),
            )
            .changed()
        {
            state.params.seed.set(seed);
        }

        ui.add_space(12.0);

        if ui
            .add_sized([ui.available_width(), 28.0], egui::Button::new("Compute"))
            .clicked()
        {
            self.outbox.push(AppMessage::Compute);
        }

        ui.add_space(8.0);
        ui.weak("submits via Signal<WorkRequest> → AsyncDispatcher → Tokio task");
    }
}
