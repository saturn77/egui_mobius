use serde::{Deserialize, Serialize};
use crate::logger::{LogColors, ButtonColors};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub slider_value: f32,
    pub combo_value: String,
    pub time_format: String,
    pub colors: LogColors,
    pub button_colors: ButtonColors,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            slider_value: 0.5,
            combo_value: "Option A".to_string(),
            time_format: "24h".to_string(),
            colors: LogColors::default(),
            button_colors: ButtonColors::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClockMessage {
    TimeUpdated(String),
}

#[derive(Debug, Clone)]
pub enum Event {
    SliderChanged(f32),
    ComboSelected(String),
}

#[derive(Debug, Clone)]
pub enum Response {
    SliderProcessed(f32),
    ComboProcessed(String),
}
