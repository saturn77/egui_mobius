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
