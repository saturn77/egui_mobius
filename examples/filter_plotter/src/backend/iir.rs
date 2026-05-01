//! In-process IIR filter backend.
//!
//! Generates a 50 Hz sine + 200 kHz tone "noise" at the configured
//! sample rate, applies a Butterworth biquad lowpass, returns both
//! traces.

use super::{BackendKind, FilterParams, Traces};

/// A direct-form-II-transposed biquad section.
///
/// The transposed topology has lower numerical sensitivity to coefficient
/// quantization than the direct form, which matters once the cutoff is
/// far below the sample rate (here: 1 kHz cutoff at 1 MHz sample rate
/// puts the filter in a corner of the unit circle where coefficient
/// precision starts to matter).
struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    /// Build a Butterworth-Q lowpass via the bilinear transform.
    fn lowpass(cutoff_hz: f32, sample_rate_hz: f32) -> Self {
        let q = std::f32::consts::FRAC_1_SQRT_2; // 0.7071… → maximally flat
        let omega = 2.0 * std::f32::consts::PI * cutoff_hz / sample_rate_hz;
        let cos_w = omega.cos();
        let alpha = omega.sin() / (2.0 * q);

        let b0 = (1.0 - cos_w) / 2.0;
        let b1 = 1.0 - cos_w;
        let b2 = (1.0 - cos_w) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x + self.z2 - self.a1 * y;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }
}

/// In-process implementation of `BackendKind`.
///
/// Replace this struct with `SerialPort` (or whatever your hardware
/// connector is) and the rest of the app — settings panel, plot panel,
/// terminal panel, dispatcher wiring — does not change.
pub struct InProcessIir;

impl InProcessIir {
    pub fn new() -> Self {
        Self
    }
}

impl BackendKind for InProcessIir {
    type Sample = f32;

    fn name(&self) -> &'static str {
        "in-process IIR"
    }

    fn run(&mut self, params: &FilterParams) -> Traces<f32> {
        let n = params.num_samples();
        let sr = params.sample_rate_hz;
        let signal_w = 2.0 * std::f32::consts::PI * params.signal_freq_hz;
        let noise_w = 2.0 * std::f32::consts::PI * params.noise_freq_hz;

        let mut filter = Biquad::lowpass(params.cutoff_hz, sr);

        let mut time = Vec::with_capacity(n);
        let mut input = Vec::with_capacity(n);
        let mut filtered = Vec::with_capacity(n);

        let noise_amp = params.noise_amplitude;
        for i in 0..n {
            let t = i as f32 / sr;
            let raw = (signal_w * t).sin() + noise_amp * (noise_w * t).sin();
            let out = filter.process(raw);

            time.push(t as f64);
            input.push(raw);
            filtered.push(out);
        }

        Traces {
            time,
            input,
            filtered,
        }
    }
}
