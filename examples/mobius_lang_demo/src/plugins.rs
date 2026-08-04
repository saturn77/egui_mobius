//! Two citizen plugins wrapping real ecosystem crates — the worked
//! examples for `PLUGINS.md`.
//!
//! Each plugin is a small struct that:
//!   1. implements `egui_citizen::Citizen` (identity + lifecycle),
//!   2. implements `mobius_lang_host::CitizenView` (per-frame draw),
//!   3. has a `build(name, bindings)` constructor that pulls the shared
//!      `Dynamic<T>` handles it needs out of the DSL-wired `BindingSet` and
//!      hands them to the real crate.
//!
//! The `.mobius` file instantiates them by name (`PlotPanel`, `LensLogger`)
//! and binds their ports; nothing here knows about the file.

use eframe::egui;
use egui_citizen::{Citizen, CitizenId, CitizenState};
use egui_lens::{LogColors, ReactiveEventLogger, ReactiveEventLoggerState};
use egui_mobius_reactive::Dynamic;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use mobius_lang::execute::{BindingSet, Watch};
use mobius_lang_host::CitizenView;

// ----------------------------------------------------------------------
// LensLogger — wraps egui_lens::ReactiveEventLogger
// ----------------------------------------------------------------------

/// A real `egui_lens` logger, driven by the values it observes in the DSL
/// graph. It owns its log buffer (legitimate internal state) and logs a
/// line whenever a bound value changes.
pub struct LensLogger {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    log_state: Dynamic<ReactiveEventLoggerState>,
    colors: Dynamic<LogColors>,
    amplitude: Option<Watch<f32>>,
    trace: Option<Watch<Vec<f32>>>,
    last_amplitude: f32,
    last_trace_len: Option<usize>,
}

impl LensLogger {
    /// The plugin constructor. `bindings` are this instance's wired ports;
    /// here they are the observe-only (`in`) views of the Bench interface.
    pub fn build(name: &str, bindings: &BindingSet) -> Box<dyn CitizenView> {
        let log_state = Dynamic::new(ReactiveEventLoggerState::new());
        let colors = Dynamic::new(LogColors::default());
        ReactiveEventLogger::with_colors(&log_state, &colors)
            .log_info("LensLogger constructed by mobius_lang — real egui_lens citizen");
        Box::new(Self {
            citizen_id: CitizenId::new(name.to_owned()),
            citizen_state: CitizenState::new(),
            log_state,
            colors,
            amplitude: bindings.read::<f32>("bench.amplitude"),
            trace: bindings.read::<Vec<f32>>("bench.trace"),
            last_amplitude: f32::NAN,
            last_trace_len: None,
        })
    }

    /// Log a line for each observed value that changed since last frame —
    /// the citizen reacting to shared state, all one-writer clean.
    fn log_changes(&mut self) {
        let logger = ReactiveEventLogger::with_colors(&self.log_state, &self.colors);
        if let Some(amplitude) = &self.amplitude {
            let value = amplitude.get();
            if value != self.last_amplitude {
                self.last_amplitude = value;
                logger.log_info(&format!("amplitude → {value:.2}"));
            }
        }
        if let Some(trace) = &self.trace {
            let len = trace.get().len();
            if self.last_trace_len != Some(len) {
                self.last_trace_len = Some(len);
                logger.log_info(&format!("trace → {len} samples"));
            }
        }
    }
}

impl Citizen for LensLogger {
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

impl CitizenView for LensLogger {
    fn ui(&mut self, ui: &mut egui::Ui) {
        self.log_changes();
        ReactiveEventLogger::with_colors(&self.log_state, &self.colors).show(ui);
    }
}

// ----------------------------------------------------------------------
// PlotPanel — wraps egui_plot
// ----------------------------------------------------------------------

/// A real `egui_plot` line plot of the observed `trace` value.
pub struct PlotPanel {
    citizen_id: CitizenId,
    citizen_state: CitizenState,
    trace: Option<Watch<Vec<f32>>>,
    amplitude: Option<Watch<f32>>,
}

impl PlotPanel {
    pub fn build(name: &str, bindings: &BindingSet) -> Box<dyn CitizenView> {
        Box::new(Self {
            citizen_id: CitizenId::new(name.to_owned()),
            citizen_state: CitizenState::new(),
            trace: bindings.read::<Vec<f32>>("bench.trace"),
            amplitude: bindings.read::<f32>("bench.amplitude"),
        })
    }
}

impl Citizen for PlotPanel {
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

impl CitizenView for PlotPanel {
    fn ui(&mut self, ui: &mut egui::Ui) {
        let samples = self.trace.as_ref().map(|w| w.get()).unwrap_or_default();
        let amplitude = self.amplitude.as_ref().map(|w| w.get()).unwrap_or(1.0);
        let bound = amplitude.abs().max(1.0) as f64 * 1.1;
        Plot::new("bench_plot")
            .legend(Legend::default())
            .include_y(-bound)
            .include_y(bound)
            .show(ui, |plot_ui| {
                if samples.len() < 2 {
                    return;
                }
                let points: PlotPoints = samples
                    .iter()
                    .enumerate()
                    .map(|(i, &y)| [i as f64, y as f64])
                    .collect();
                plot_ui
                    .line(Line::new("trace", points).color(egui::Color32::from_rgb(122, 162, 247)));
            });
    }
}
