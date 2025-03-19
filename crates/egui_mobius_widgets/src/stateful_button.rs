//! A stateful button widget that maintains its state between frames.
//!
//! The `StatefulButton` is designed for toggle-like behavior with the following features:
//! - Maintains state (started/stopped) between frames
//! - Customizable colors for each state
//! - Adjustable corner rounding and margins
//! - Hover effect with outer stroke
//! - Default implementation for easy instantiation
//!
//! # Example
//!
//! ```rust,no_run
//! use egui_mobius_widgets::StatefulButton;
//! use eframe::egui;
//!
//! fn ui_example(ui: &mut egui::Ui) {
//!     let mut button = StatefulButton::new()
//!         .run_color(egui::Color32::GREEN)
//!         .stop_color(egui::Color32::RED)
//!         .rounding(8.0);
//!
//!     if button.show(ui).clicked() {
//!         println!("Button state: {}", button.is_started());
//!     }
//! }
//! ```

use egui::{Response, Ui, Color32, CornerRadius, Stroke, Vec2};
use egui::epaint::StrokeKind;

/// A button that maintains its state (started/stopped) and changes appearance accordingly.
///
/// The button supports:
/// - Toggle state (started/stopped)
/// - Different colors for each state
/// - Corner rounding
/// - Margin and minimum size settings
/// - Automatic state persistence between frames
/// 
/// Useful for buttons that need to toggle between two states, such as start/stop or on/off. 
/// 
/// # Example
/// 
/// ```rust
/// use egui_mobius_widgets::StatefulButton;
/// use eframe::egui;
/// 
#[derive(Debug)]
pub struct StatefulButton {
    started: bool,
    margin: Vec2,
    rounding: f32,
    min_size: Vec2,
    run_color: Color32,
    stop_color: Color32,
}

impl Default for StatefulButton {
    fn default() -> Self {
        Self::new()
    }
}

impl StatefulButton {
    /// Creates a new stateful button with default styling.
    ///
    /// # Default Values
    ///
    /// * `started` - false (STOP state)
    /// * `margin` - Vec2::new(8.5, 4.25)
    /// * `rounding` - 8.0 pixels
    /// * `min_size` - Vec2::ZERO
    /// * `run_color` - Color32::GREEN
    /// * `stop_color` - Color32::RED
    pub fn new() -> Self {
        Self {
            started: false,
            margin: Vec2::new(8.5, 4.25),
            rounding: 8.0,
            min_size: Vec2::new(0.0, 0.0),
            run_color: Color32::GREEN,
            stop_color: Color32::RED,
        }
    }

    /// Sets the margin (space around the button).
    ///
    /// # Arguments
    ///
    /// * `margin` - A Vec2 where x is horizontal margin and y is vertical margin
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn margin(mut self, margin: Vec2) -> Self {
        self.margin = margin;
        self
    }

    /// Sets the radius for rounding the button's corners.
    ///
    /// # Arguments
    ///
    /// * `rounding` - The corner radius in pixels
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn rounding(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self
    }

    /// Sets the minimum size of the button.
    ///
    /// # Arguments
    ///
    /// * `min_size` - A Vec2 where x is minimum width and y is minimum height
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Sets the color used when the button is in the RUN state.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to use for the RUN state
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn run_color(mut self, color: Color32) -> Self {
        self.run_color = color;
        self
    }

    /// Sets the color used when the button is in the STOP state.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to use for the STOP state
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn stop_color(mut self, color: Color32) -> Self {
        self.stop_color = color;
        self
    }

    /// Shows the button in the UI and returns the response.
    ///
    /// The button's text will automatically switch between "RUN" and "STOP"
    /// based on its current state. Clicking the button will toggle its state.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI to add the button to
    ///
    /// # Returns
    ///
    /// Returns an egui::Response that can be used to check for clicks and hover state
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        ui.add_space(self.margin.y);
        let response = ui.horizontal(|ui| {
            ui.add_space(self.margin.x);
            let text = if self.started { "RUN" } else { "STOP" };
            let color = if self.started { self.run_color } else { self.stop_color };
            let button = egui::Button::new(text)
                .fill(egui::Color32::TRANSPARENT)
                .corner_radius(CornerRadius::from(self.rounding))
                .min_size(self.min_size);

            let response = ui.add(button);

            if response.hovered() {
                ui.painter().rect_stroke(
                    response.rect,
                    CornerRadius::from(self.rounding),
                    Stroke::new(1.0, color.gamma_multiply(1.2)),
                    StrokeKind::Outside,
                );
            } else {
                ui.painter().rect_stroke(
                    response.rect,
                    CornerRadius::from(self.rounding),
                    Stroke::new(1.0, color),
                    StrokeKind::Outside,
                );
            }

            ui.add_space(self.margin.x);
            response
        }).inner;

        if response.clicked() {
            self.started = !self.started;
        }

        response
    }

    /// Returns the current state of the button.
    ///
    /// # Returns
    ///
    /// * `true` - Button is in RUN state
    /// * `false` - Button is in STOP state
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Sets the current state of the button.
    ///
    /// # Arguments
    ///
    /// * `started` - The new state to set:
    ///   * `true` - Set to RUN state
    ///   * `false` - Set to STOP state
    pub fn set_started(&mut self, started: bool) {
        self.started = started;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stateful_button_creation() {
        let button = StatefulButton::new()
            .run_color(Color32::RED)
            .stop_color(Color32::BLUE)
            .rounding(10.0)
            .margin(Vec2::new(10.0, 5.0));

        assert!(!button.is_started());
        assert_eq!(button.run_color, Color32::RED);
        assert_eq!(button.stop_color, Color32::BLUE);
        assert_eq!(button.rounding, 10.0);
        assert_eq!(button.margin, Vec2::new(10.0, 5.0));
    }

    #[test]
    fn test_stateful_button_default() {
        let button = StatefulButton::default();
        assert!(!button.is_started());
        assert_eq!(button.margin, Vec2::new(8.5, 4.25));
        assert_eq!(button.rounding, 8.0);
        assert_eq!(button.min_size, Vec2::new(0.0, 0.0));
        assert_eq!(button.run_color, Color32::GREEN);
        assert_eq!(button.stop_color, Color32::RED);
    }

    #[test]
    fn test_stateful_button_state_changes() {
        let mut button = StatefulButton::new();
        assert!(!button.is_started());

        button.set_started(true);
        assert!(button.is_started());

        button.set_started(false);
        assert!(!button.is_started());
    }

    #[test]
    fn test_stateful_button_min_size() {
        let button = StatefulButton::new()
            .min_size(Vec2::new(100.0, 50.0));
        assert_eq!(button.min_size, Vec2::new(100.0, 50.0));
    }
}
