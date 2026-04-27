//! Backend abstraction: a `BackendKind` runs a snapshot of input + filtered samples.
//!
//! The tutorial ships `iir::InProcessIir` as the working implementation —
//! generates a noisy sinewave locally and applies a digital lowpass IIR
//! filter to it. The same trait shape would fit a serial-port-backed
//! variant (`SerialPort` impl below the `iir` module) where the input
//! samples are read off a USB serial port instead of generated.

pub mod iir;

/// Parameters captured at "Generate" time — a snapshot of the reactive
/// fields on `SharedState::params` so the backend has a stable, owned
/// view of what to compute.
#[derive(Debug, Clone, Copy)]
pub struct FilterParams {
    pub signal_freq_hz: f32,
    pub noise_freq_hz: f32,
    pub cutoff_hz: f32,
    pub sample_rate_hz: f32,
    pub duration_ms: f32,
}

impl FilterParams {
    pub fn num_samples(&self) -> usize {
        (self.sample_rate_hz * self.duration_ms / 1000.0).round() as usize
    }
}

/// One pair of traces resulting from a Generate run.
#[derive(Debug, Clone, Default)]
pub struct Traces {
    pub time: Vec<f64>,      // seconds
    pub input: Vec<f64>,     // raw noisy signal
    pub filtered: Vec<f64>,  // lowpass output
}

impl Traces {
    pub fn is_empty(&self) -> bool {
        self.time.is_empty()
    }
}

/// What every backend variant must do: turn a parameter snapshot into a
/// pair of traces (input, filtered).
pub trait BackendKind {
    fn run(&mut self, params: &FilterParams) -> Traces;
    fn name(&self) -> &'static str;
}
