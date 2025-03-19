//! A customizable button widget with enhanced styling options.
//!
//! The `StyledButton` provides a flexible button implementation with the following features:
//! - Customizable hover and normal colors
//! - Adjustable corner rounding
//! - Configurable margins and minimum size
//! - Hover effect with outer stroke
//!
//! # Example
//!
//! ```rust,no_run
//! use egui_mobius_widgets::StyledButton;
//! use eframe::egui;
//!
//! fn ui_example(ui: &mut egui::Ui) {
//!     let button = StyledButton::new("Click me")
//!         .hover_color(egui::Color32::RED)
//!         .normal_color(egui::Color32::BLUE)
//!         .rounding(8.0)
//!         .margin(egui::Vec2::new(10.0, 5.0));
//!
//!     if button.show(ui).clicked() {
//!         println!("Button clicked!");
//!     }
//! }
//! ```

use egui::{Response, Ui, Color32, CornerRadius, Stroke, Vec2};
use egui::epaint::StrokeKind;

/// A styled button with customizable appearance and hover effects.
///
/// The button supports:
/// - Custom text
/// - Hover and normal colors
/// - Corner rounding
/// - Margin and minimum size settings
#[derive(Debug)]
pub struct StyledButton {
    text: String,
    hover_color: Color32,
    normal_color: Color32,
    text_color: Color32,
    rounding: f32,
    margin: Vec2,
    min_size: Vec2,
}

impl Default for StyledButton {
    fn default() -> Self {
        Self::new("Button")
    }
}

impl StyledButton {
    /// Creates a new styled button with the given text and default styling.
    ///
    /// # Arguments
    ///
    /// * `text` - The text to display on the button
    ///
    /// # Default Values
    ///
    /// * `hover_color` - Light blue (RGB: 100, 200, 255)
    /// * `normal_color` - Gray (RGB: 128, 128, 128)
    /// * `rounding` - 5.0 pixels
    /// * `margin` - Vec2::new(10.0, 5.0)
    /// * `min_size` - Vec2::ZERO
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            hover_color: Color32::from_rgb(100, 200, 255),  // Default to light blue
            normal_color: Color32::from_gray(128),          // Default to gray
            text_color: Color32::WHITE,                    // Default to white text
            rounding: 5.0,
            margin: Vec2::new(10.0, 5.0),
            min_size: Vec2::new(0.0, 0.0),
        }
    }

    /// Sets the color of the button's border when hovered.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to use for the hover effect
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn hover_color(mut self, color: Color32) -> Self {
        self.hover_color = color;
        self
    }

    /// Sets the color of the button's border in its normal state.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to use for the normal state
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn normal_color(mut self, color: Color32) -> Self {
        self.normal_color = color;
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

    /// Sets the text color of the button.
    ///
    /// # Arguments
    ///
    /// * `color` - The color to use for the button text
    ///
    /// # Returns
    ///
    /// Returns self for method chaining
    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = color;
        self
    }

    /// Shows the button in the UI and returns the response.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI to add the button to
    ///
    /// # Returns
    ///
    /// Returns an egui::Response that can be used to check for clicks and hover state
    pub fn show(self, ui: &mut Ui) -> Response {
        let Self { text, hover_color, normal_color, text_color, rounding, margin, min_size } = self;

        ui.add_space(margin.y);
        let response = ui.horizontal(|ui| {
            ui.add_space(margin.x);
            let button = egui::Button::new(egui::RichText::new(&text).color(text_color))
                .fill(egui::Color32::TRANSPARENT)
                .corner_radius(CornerRadius::from(rounding))
                .min_size(min_size);

            let response = ui.add(button);

            if response.hovered() {
                ui.painter().rect_stroke(
                    response.rect,
                    CornerRadius::from(rounding),
                    Stroke::new(1.0, hover_color),
                    StrokeKind::Outside,
                );
            } else {
                ui.painter().rect_stroke(
                    response.rect,
                    CornerRadius::from(rounding),
                    Stroke::new(1.0, normal_color),
                    StrokeKind::Outside,
                );
            }

            ui.add_space(margin.x);
            response
        }).inner;



        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_styled_button_creation() {
        let button = StyledButton::new("Test")
            .hover_color(Color32::RED)
            .normal_color(Color32::BLUE)
            .text_color(Color32::BLACK)
            .rounding(10.0)
            .margin(Vec2::new(10.0, 5.0));

        assert_eq!(button.text, "Test");
        assert_eq!(button.hover_color, Color32::RED);
        assert_eq!(button.normal_color, Color32::BLUE);
        assert_eq!(button.text_color, Color32::BLACK);
        assert_eq!(button.rounding, 10.0);
        assert_eq!(button.margin, Vec2::new(10.0, 5.0));
    }
}
