//! Two stacked plots — input on top, filtered output below — with
//! linked X axes via `egui_plot::LinkedAxisGroup`. Pan / zoom on either
//! plot drives both, matplotlib-style.

use eframe::egui;
use egui_citizen::{CitizenId, CitizenState};
use egui_plot::{Line, Plot, PlotPoints};

use crate::state::SharedState;

/// Decimation step for plotting. The backend computes at 1 MHz × 100 ms
/// = 100,000 samples per trace; egui_plot can render that, but every
/// 50th sample is visually identical to the eye and much faster.
const PLOT_STRIDE: usize = 50;

pub struct PlotPanel {
    pub citizen_id: CitizenId,
    pub citizen_state: CitizenState,
}

impl PlotPanel {
    pub fn new(citizen_state: CitizenState) -> Self {
        Self {
            citizen_id: CitizenId::new(crate::tabs::PLOT_ID),
            citizen_state,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        let traces = state.traces.get();

        if traces.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("Click \"Generate\" in the Settings panel to compute traces.");
            });
            return;
        }

        let total = ui.available_height();
        let half = (total - 8.0).max(120.0) / 2.0;

        ui.allocate_ui(egui::vec2(ui.available_width(), half), |ui| {
            Plot::new("input_plot")
                .link_axis(state.plot_link, [true, false])
                .height(half)
                .show(ui, |plot_ui| {
                    let pts: PlotPoints = traces
                        .time
                        .iter()
                        .zip(traces.input.iter())
                        .step_by(PLOT_STRIDE)
                        .map(|(&t, &y)| [t, y])
                        .collect();
                    plot_ui.line(Line::new("input", pts).name("input"));
                });
        });

        ui.add_space(4.0);

        ui.allocate_ui(egui::vec2(ui.available_width(), half), |ui| {
            Plot::new("output_plot")
                .link_axis(state.plot_link, [true, false])
                .height(half)
                .show(ui, |plot_ui| {
                    let pts: PlotPoints = traces
                        .time
                        .iter()
                        .zip(traces.filtered.iter())
                        .step_by(PLOT_STRIDE)
                        .map(|(&t, &y)| [t, y])
                        .collect();
                    plot_ui.line(Line::new("filtered", pts).name("filtered"));
                });
        });
    }
}
