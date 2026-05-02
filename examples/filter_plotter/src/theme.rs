//! Visual theme + font scaling, separated from the eframe shell so the
//! main.rs doesn't fill up with style configuration.

use eframe::egui::{self, Color32, Visuals};

/// Apply a dark theme with slightly toned-down chrome — enough contrast
/// to read plot traces without bright panel backgrounds competing.
pub fn apply_visuals(ctx: &egui::Context) {
    let mut v = Visuals::dark();
    v.panel_fill = Color32::from_rgb(0x1a, 0x1b, 0x26);
    v.window_fill = Color32::from_rgb(0x1f, 0x21, 0x2e);
    v.extreme_bg_color = Color32::from_rgb(0x16, 0x16, 0x1e);
    ctx.set_visuals(v);
}
