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
    pub noise_amplitude: f32,
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
///
/// `T` is the sample type. The in-process IIR backend uses `f32`; a
/// serial-port backend feeding raw ADC counts could use `i16` or `i32`
/// without a lossy upcast at the boundary. Time stays `f64` regardless
/// — timestamps are the same kind of value across all backends.
#[derive(Debug, Clone)]
pub struct Traces<T> {
    pub time: Vec<f64>,   // seconds
    pub input: Vec<T>,    // raw noisy signal
    pub filtered: Vec<T>, // lowpass output
}

impl<T> Default for Traces<T> {
    fn default() -> Self {
        Self {
            time: Vec::new(),
            input: Vec::new(),
            filtered: Vec::new(),
        }
    }
}

impl<T> Traces<T> {
    pub fn is_empty(&self) -> bool {
        self.time.is_empty()
    }
}

/// What every backend variant must do: turn a parameter snapshot into a
/// pair of traces (input, filtered).
///
/// `Sample` is the per-trace sample type. Pick the type that matches
/// where the data comes from (`f32` for the in-process IIR; `i16` for a
/// 16-bit ADC over serial; `f64` for a high-precision simulator).
pub trait BackendKind {
    type Sample;
    fn run(&mut self, params: &FilterParams) -> Traces<Self::Sample>;
    fn name(&self) -> &'static str;
}
