//! Shared application state passed by reference to every panel.

use eframe::egui;
use egui_lens::{LogColors, ReactiveEventLoggerState};
use egui_mobius_reactive::Dynamic;

use crate::backend::{FilterParams, Traces};

/// Reactive parameters edited by the settings panel and consumed by the
/// backend at Generate time.
pub struct ParamsState {
    pub signal_freq_hz: Dynamic<f32>,
    pub noise_freq_hz: Dynamic<f32>,
    pub noise_amplitude: Dynamic<f32>,
    pub cutoff_hz: Dynamic<f32>,
    pub sample_rate_hz: Dynamic<f32>,
    pub duration_ms: Dynamic<f32>,
}

impl ParamsState {
    pub fn defaults() -> Self {
        Self {
            signal_freq_hz: Dynamic::new(50.0),
            noise_freq_hz: Dynamic::new(200_000.0),
            noise_amplitude: Dynamic::new(0.5),
            cutoff_hz: Dynamic::new(1_000.0),
            sample_rate_hz: Dynamic::new(1_000_000.0),
            duration_ms: Dynamic::new(100.0),
        }
    }

    /// Capture an owned snapshot for the backend.
    pub fn snapshot(&self) -> FilterParams {
        FilterParams {
            signal_freq_hz: self.signal_freq_hz.get(),
            noise_freq_hz: self.noise_freq_hz.get(),
            noise_amplitude: self.noise_amplitude.get(),
            cutoff_hz: self.cutoff_hz.get(),
            sample_rate_hz: self.sample_rate_hz.get(),
            duration_ms: self.duration_ms.get(),
        }
    }
}

/// Everything the panels read from. The plot panel reads `traces` and
/// `plot_link`, the terminal reads `log`, the settings panel reads and
/// writes `params`.
pub struct SharedState {
    pub params: ParamsState,
    pub traces: Dynamic<Traces<f32>>,
    pub log: Dynamic<ReactiveEventLoggerState>,
    pub log_colors: Dynamic<LogColors>,
    /// Linked-axis group id — both Plot widgets pass this same Id to
    /// `Plot::link_axis(...)` so pan/zoom on either propagates to the
    /// other (matplotlib-style linked subplots).
    pub plot_link: egui::Id,
    /// When true, the input plot also shows the filtered trace as a
    /// green overlay so the two can be compared without zooming
    /// between subplots.
    pub overlay_filtered: Dynamic<bool>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            params: ParamsState::defaults(),
            traces: Dynamic::new(Traces::<f32>::default()),
            log: Dynamic::new(ReactiveEventLoggerState::new()),
            log_colors: {
                let mut colors = LogColors::default();
                // Color the citizen lifecycle stream distinctly
                colors.set_custom_color("citizen", egui::Color32::from_rgb(100, 200, 255));
                colors.set_custom_color("backend", egui::Color32::from_rgb(140, 220, 140));
                Dynamic::new(colors)
            },
            plot_link: egui::Id::new("filter_plotter::axis_link"),
            overlay_filtered: Dynamic::new(false),
        }
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}
