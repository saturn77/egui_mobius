use egui::{Color32, Margin, Response, Stroke, Ui, Widget};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonState {
    Idle,
    Hovered,
    Pressed,
    Clicked,
}

#[derive(Debug, Default, Clone)]
pub struct ButtonStyle {
    pub stroke_size          : Option<u8>,
    pub stroke_size_on_hover : Option<u8>,
    pub stroke_color         : Option<Color32>,
    pub hovered_color        : Option<Color32>,
    pub corner_radius        : Option<u8>,
    pub inner_margin         : Option<Margin>,
}

pub struct StatefulButton {
    pub id    : usize,
    pub label : String,
    pub state : ButtonState,
    pub style : ButtonStyle,
}

impl StatefulButton {
    pub fn new(assigned_id: usize, label: &str, style: ButtonStyle) -> Self {
        Self {
            id    : assigned_id,
            label : label.to_string(),
            state : ButtonState::Idle,
            style,
        }
    }

    pub fn next_state_logic(&mut self, response: &Response) {
        if response.clicked() {
            self.state = ButtonState::Clicked;
        } else if response.is_pointer_button_down_on() {
            self.state = ButtonState::Pressed;
        } else if response.hovered() {
            self.state = ButtonState::Hovered;
        } else {
            self.state = ButtonState::Idle;
        }
    }

    pub fn apply_style(&mut self, style: &mut egui::Style) {
        match self.state {
            ButtonState::Clicked => style.visuals.widgets.inactive.bg_fill = egui::Color32::RED,
            ButtonState::Pressed => style.visuals.widgets.inactive.bg_fill = egui::Color32::BLUE,
            ButtonState::Hovered => {
                style.visuals.widgets.hovered.bg_fill = self.style.hovered_color.unwrap_or(egui::Color32::GRAY);
                self.style.stroke_size = Some(4);
            },
            ButtonState::Idle => {
                style.visuals.widgets.inactive.bg_fill = egui::Color32::GREEN;
                self.style.stroke_size = Some(1);
            },
        }
    }

    pub fn ui_with_style(&mut self, ui: &mut Ui) -> Response {
        let mut style = egui::Style::default();
        self.apply_style(&mut style);

        let frame = egui::Frame::new()
            .fill(style.visuals.widgets.hovered.bg_fill)
            .corner_radius(self.style.corner_radius.unwrap_or(10) as f32)
            .stroke(Stroke::new(self.style.stroke_size.unwrap_or(1) as f32, self.style.stroke_color.unwrap_or(egui::Color32::GREEN)))
            .inner_margin(self.style.inner_margin.unwrap_or(Margin::same(5)));

        frame.show(ui, |ui| {
            self.ui(ui)
        }).inner
    }
}

impl Widget for &mut StatefulButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let button = egui::Button::new(&self.label);
        let response = ui.add(button);

        self.next_state_logic(&response);
        response
    }
}