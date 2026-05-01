//! Two stacked plots — input on top, filtered output below — with
//! linked X axes via `egui_plot::LinkedAxisGroup`. Pan / zoom on either
//! plot drives both, matplotlib-style.

use eframe::egui;
use egui_citizen::{CitizenId, CitizenState};
use egui_plot::{Line, Plot, PlotPoints};

use crate::state::SharedState;

/// Decimation strides chosen separately for input vs filtered output.
///
/// **Why not the same stride for both?** If you stride too aggressively
/// on the *input* trace, the high-frequency noise gets aliased back to
/// zero on the displayed plot — at 1 MHz sample rate, the 200 kHz
/// noise has a period of exactly 5 samples, so a stride of 50 samples
/// the noise at every 10th period (same phase every time, which
/// happens to be sin(0) = 0). The noise is still there in the data;
/// it just lands invisible. Use stride 1 (every sample) for the input
/// plot so the noise renders correctly.
///
/// The *filtered* trace is smooth — anything above the 1 kHz cutoff is
/// attenuated. Stride 50 is plenty there and keeps the renderer happy.
const INPUT_STRIDE: usize = 1;
const FILTERED_STRIDE: usize = 50;

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

        let overlay = state.overlay_filtered.get();

        ui.allocate_ui(egui::vec2(ui.available_width(), half), |ui| {
            Plot::new("input_plot")
                .link_axis(state.plot_link, [true, false])
                .height(half)
                .show(ui, |plot_ui| {
                    let pts: PlotPoints = traces
                        .time
                        .iter()
                        .zip(traces.input.iter())
                        .step_by(INPUT_STRIDE)
                        .map(|(&t, &y)| [t, y as f64])
                        .collect();
                    plot_ui.line(Line::new("input", pts).name("input"));

                    if overlay {
                        let pts: PlotPoints = traces
                            .time
                            .iter()
                            .zip(traces.filtered.iter())
                            .step_by(FILTERED_STRIDE)
                            .map(|(&t, &y)| [t, y as f64])
                            .collect();
                        plot_ui.line(
                            Line::new("filtered", pts)
                                .color(egui::Color32::from_rgb(0x90, 0xe0, 0x90))
                                .name("filtered"),
                        );
                    }
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
                        .step_by(FILTERED_STRIDE)
                        .map(|(&t, &y)| [t, y as f64])
                        .collect();
                    plot_ui.line(Line::new("filtered", pts).name("filtered"));
                });
        });
    }
}
