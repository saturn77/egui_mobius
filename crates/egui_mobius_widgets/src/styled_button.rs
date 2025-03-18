use egui::{Response, Ui, Color32, CornerRadius, Stroke, Vec2};
use egui::epaint::StrokeKind;

/// A styled button that changes border color on hover
#[derive(Debug)]
pub struct StyledButton {
    text: String,
    hover_color: Color32,
    normal_color: Color32,
    rounding: f32,
    margin: Vec2,
    min_size: Vec2,
}

impl StyledButton {
    /// Create a new styled button
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            hover_color: Color32::from_rgb(100, 200, 255),  // Default to light blue
            normal_color: Color32::from_gray(128),          // Default to gray
            rounding: 5.0,
            margin: Vec2::new(10.0, 5.0),
            min_size: Vec2::new(0.0, 0.0),
        }
    }

    /// Set the hover color
    pub fn hover_color(mut self, color: Color32) -> Self {
        self.hover_color = color;
        self
    }

    /// Set the normal color
    pub fn normal_color(mut self, color: Color32) -> Self {
        self.normal_color = color;
        self
    }

    /// Set the corner rounding
    pub fn rounding(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self
    }

    /// Set the margin - space around the button ()
    pub fn margin(mut self, margin: Vec2) -> Self {
        self.margin = margin;
        self
    }

    /// Set the minimum size of the button
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Show the button in the UI
    pub fn show(self, ui: &mut Ui) -> Response {
        let Self { text, hover_color, normal_color, rounding, margin, min_size } = self;

        ui.add_space(margin.y);
        let response = ui.horizontal(|ui| {
            ui.add_space(margin.x);
            let response = ui.add(
                egui::Button::new(&text)
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(1.0, normal_color))
                    .corner_radius(CornerRadius::from(rounding))
                    .min_size(min_size)
            );
            ui.add_space(margin.x);
            response
        }).inner;

        // Draw hover effect
        if response.hovered() {
            ui.painter().rect_stroke(
                response.rect,
                CornerRadius::from(rounding),
                Stroke::new(2.0, hover_color),
                StrokeKind::Outside,  // Draw stroke outside the rect
            );
        }

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
            .rounding(10.0)
            .margin(Vec2::new(10.0, 5.0));

        assert_eq!(button.text, "Test");
        assert_eq!(button.hover_color, Color32::RED);
        assert_eq!(button.normal_color, Color32::BLUE);
        assert_eq!(button.rounding, 10.0);
        assert_eq!(button.margin, Vec2::new(10.0, 5.0));
    }
}
