use egui::{Response, Ui, Color32, CornerRadius, Stroke, Vec2};
use egui::epaint::StrokeKind;

/// A button that maintains its state (started/stopped) and changes appearance accordingly
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
    /// Create a new stateful button
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

    /// Set the margin around the button
    pub fn margin(mut self, margin: Vec2) -> Self {
        self.margin = margin;
        self
    }

    /// Set the corner rounding of the button
    pub fn rounding(mut self, rounding: f32) -> Self {
        self.rounding = rounding;
        self
    }

    /// Set the minimum size of the button
    pub fn min_size(mut self, min_size: Vec2) -> Self {
        self.min_size = min_size;
        self
    }

    /// Set the color for RUN state
    pub fn run_color(mut self, color: Color32) -> Self {
        self.run_color = color;
        self
    }

    /// Set the color for STOP state
    pub fn stop_color(mut self, color: Color32) -> Self {
        self.stop_color = color;
        self
    }

    /// Show the button in the UI
    pub fn show(&mut self, ui: &mut Ui) -> Response {
        ui.add_space(self.margin.y);
        let response = ui.horizontal(|ui| {
            ui.add_space(self.margin.x);
            let text = if self.started { "RUN" } else { "STOP" };
            let response = ui.add(
                egui::Button::new(text)
                    .fill(Color32::TRANSPARENT)
                    .stroke(Stroke::new(
                        1.0,
                        if self.started {
                            self.run_color
                        } else {
                            self.stop_color
                        },
                    ))
                    .corner_radius(CornerRadius::from(self.rounding))
                    .min_size(self.min_size)
            );
            ui.add_space(self.margin.x);
            response
        }).inner;

        if response.clicked() {
            self.started = !self.started;
        }

        // Draw hover effect
        if response.hovered() {
            ui.painter().rect_stroke(
                response.rect,
                CornerRadius::from(self.rounding),
                Stroke::new(
                    2.0,
                    if self.started {
                        self.run_color
                    } else {
                        self.stop_color
                    },
                ),
                StrokeKind::Outside,
            );
        }

        response
    }

    /// Get the current state of the button
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Set the current state of the button
    pub fn set_started(&mut self, started: bool) {
        self.started = started;
    }
}
