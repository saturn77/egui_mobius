//! Tokyo Night Storm — the bench-app palette, ported from blaze2-gui.

use eframe::egui::{self, Color32};

pub struct TokyoNight;

impl TokyoNight {
    pub const BG: Color32 = Color32::from_rgb(0x24, 0x28, 0x3b);
    pub const BG_DARK: Color32 = Color32::from_rgb(0x1f, 0x23, 0x35);
    pub const BG_HIGHLIGHT: Color32 = Color32::from_rgb(0x2f, 0x33, 0x4d);
    pub const FG: Color32 = Color32::from_rgb(0xc0, 0xca, 0xf5);
    pub const FG_DIM: Color32 = Color32::from_rgb(0xa9, 0xb1, 0xd6);
    pub const BLUE: Color32 = Color32::from_rgb(0x7a, 0xa2, 0xf7);
    pub const CYAN: Color32 = Color32::from_rgb(0x7d, 0xcf, 0xff);
    pub const GREEN: Color32 = Color32::from_rgb(0x9e, 0xce, 0x6a);
    pub const RED: Color32 = Color32::from_rgb(0xf7, 0x76, 0x8e);
    pub const ORANGE: Color32 = Color32::from_rgb(0xff, 0x9e, 0x64);
    pub const BORDER: Color32 = Color32::from_rgb(0x3b, 0x41, 0x61);
    pub const SELECTION: Color32 = Color32::from_rgb(0x2e, 0x3c, 0x64);
}

pub fn apply(ctx: &egui::Context) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = TokyoNight::BG;
    v.window_fill = TokyoNight::BG_DARK;
    v.faint_bg_color = TokyoNight::BG_HIGHLIGHT;
    v.extreme_bg_color = TokyoNight::BG_DARK;
    v.selection.bg_fill = TokyoNight::SELECTION;
    v.selection.stroke = egui::Stroke::new(1.0, TokyoNight::BLUE);
    v.widgets.noninteractive.bg_fill = TokyoNight::BG;
    v.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, TokyoNight::FG_DIM);
    v.widgets.noninteractive.bg_stroke = egui::Stroke::new(1.0, TokyoNight::BORDER);
    v.widgets.inactive.bg_fill = TokyoNight::BG_HIGHLIGHT;
    v.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, TokyoNight::FG);
    v.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, TokyoNight::BORDER);
    v.widgets.hovered.bg_fill = TokyoNight::SELECTION;
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, TokyoNight::CYAN);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, TokyoNight::BLUE);
    v.widgets.active.bg_fill = TokyoNight::BLUE;
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, TokyoNight::BG_DARK);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, TokyoNight::CYAN);
    v.override_text_color = Some(TokyoNight::FG);
    v.hyperlink_color = TokyoNight::CYAN;
    v.warn_fg_color = TokyoNight::ORANGE;
    v.error_fg_color = TokyoNight::RED;
    v.window_stroke = egui::Stroke::new(1.0, TokyoNight::BORDER);
    ctx.set_visuals(v);
}
