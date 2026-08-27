//! Filter parameter sliders + Generate button. Pushes
//! `AppMessage::Generate` onto its outbox; main.rs drains it and
//! forwards through `settings_actions::handle` to the backend.

use eframe::egui;

use crate::messages::AppMessage;
use crate::state::SharedState;

use crate::tabs::SETTINGS_ID;
use egui_citizen::citizen_panel;

// `outbox` — outgoing app-level intents. Populated by show(); main.rs
// drains it each frame into `settings_actions::handle`.
citizen_panel!(SettingsPanel, SETTINGS_ID, outbox: Vec<AppMessage> = Vec::new());

impl SettingsPanel {
    pub fn show(&mut self, ui: &mut egui::Ui, state: &SharedState) {
        ui.heading("Filter parameters");
        ui.add_space(8.0);

        // Signal frequency
        let mut signal = state.params.signal_freq_hz.get();
        if ui
            .add(
                egui::Slider::new(&mut signal, 1.0..=500.0)
                    .text("Signal frequency (Hz)")
                    .logarithmic(true),
            )
            .changed()
        {
            state.params.signal_freq_hz.set(signal);
        }

        // Noise frequency
        let mut noise = state.params.noise_freq_hz.get();
        if ui
            .add(
                egui::Slider::new(&mut noise, 10_000.0..=400_000.0)
                    .text("Noise frequency (Hz)")
                    .logarithmic(true),
            )
            .changed()
        {
            state.params.noise_freq_hz.set(noise);
        }

        // Noise amplitude
        let mut noise_amp = state.params.noise_amplitude.get();
        if ui
            .add(
                egui::Slider::new(&mut noise_amp, 0.0..=2.0)
                    .text("Noise amplitude")
                    .fixed_decimals(2),
            )
            .changed()
        {
            state.params.noise_amplitude.set(noise_amp);
        }

        // Cutoff
        let mut cutoff = state.params.cutoff_hz.get();
        if ui
            .add(
                egui::Slider::new(&mut cutoff, 100.0..=50_000.0)
                    .text("Lowpass cutoff (Hz)")
                    .logarithmic(true),
            )
            .changed()
        {
            state.params.cutoff_hz.set(cutoff);
        }

        // Sample rate (read-only display + small range)
        let mut sr = state.params.sample_rate_hz.get();
        if ui
            .add(
                egui::Slider::new(&mut sr, 100_000.0..=2_000_000.0)
                    .text("Sample rate (Hz)")
                    .logarithmic(true),
            )
            .changed()
        {
            state.params.sample_rate_hz.set(sr);
        }

        // Duration
        let mut dur = state.params.duration_ms.get();
        if ui
            .add(egui::Slider::new(&mut dur, 10.0..=500.0).text("Duration (ms)"))
            .changed()
        {
            state.params.duration_ms.set(dur);
        }

        ui.add_space(12.0);

        // Display options.
        let mut overlay = state.overlay_filtered.get();
        if ui
            .checkbox(
                &mut overlay,
                "Overlay filtered output on input plot (green)",
            )
            .changed()
        {
            state.overlay_filtered.set(overlay);
        }

        ui.add_space(8.0);

        if ui
            .add_sized([ui.available_width(), 28.0], egui::Button::new("Generate"))
            .clicked()
        {
            self.outbox.push(AppMessage::Generate);
        }

        ui.add_space(8.0);

        let snap = state.params.snapshot();
        ui.label(format!(
            "→ {} samples ({:.0} kHz × {} ms)",
            snap.num_samples(),
            snap.sample_rate_hz / 1000.0,
            snap.duration_ms as u32,
        ));
    }
}
