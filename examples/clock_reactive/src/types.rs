use serde::{Deserialize, Serialize};
use egui::Color32;
use chrono::{DateTime, Local};
use std::{fs::OpenOptions, io::Write};

mod color32_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use eframe::egui::Color32;

    pub fn serialize<S>(color: &Color32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let rgba = [color.r(), color.g(), color.b(), color.a()];
        rgba.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Color32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let rgba = <[u8; 4]>::deserialize(deserializer)?;
        Ok(Color32::from_rgba_unmultiplied(rgba[0], rgba[1], rgba[2], rgba[3]))
    }
}

#[derive(Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub source: String,
    pub message: String,
    pub color: Option<Color32>,
}

impl PartialEq for LogEntry {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp.timestamp() == other.timestamp.timestamp() &&
        self.source == other.source &&
        self.message == other.message &&
        self.color == other.color
    }
}

impl LogEntry {
    pub fn formatted(&self) -> String {
        format!(
            "[{}] [{}] {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.source,
            self.message
        )
    }

    pub fn save_to_file(&self) -> std::io::Result<()> {
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("ui_session_log.txt")
        {
            writeln!(file, "{}", self.formatted())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub enum ClockMessage {
    TimeUpdated(()),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogColors {
    #[serde(with = "color32_serde")]
    pub clock: Color32,
    #[serde(with = "color32_serde")]
    pub slider: Color32,
    #[serde(with = "color32_serde")]
    pub option_a: Color32,
    #[serde(with = "color32_serde")]
    pub option_b: Color32,
    #[serde(with = "color32_serde")]
    pub option_c: Color32,
    #[serde(with = "color32_serde")]
    pub time_format: Color32,
    #[serde(with = "color32_serde")]
    pub custom_event: Color32,
    #[serde(with = "color32_serde")]
    pub run_stop_log: Color32,
}

impl Default for LogColors {
    fn default() -> Self {
        Self {
            clock: Color32::from_rgb(100, 200, 100),
            slider: Color32::from_rgb(200, 150, 100),
            option_a: Color32::from_rgb(100, 150, 200),
            option_b: Color32::from_rgb(150, 100, 200),
            option_c: Color32::from_rgb(200, 100, 150),
            time_format: Color32::from_rgb(180, 180, 100),
            custom_event: Color32::from_rgb(100, 180, 180),
            run_stop_log: Color32::from_rgb(180, 100, 180),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonColors {
    #[serde(with = "color32_serde")]
    pub run_state: Color32,
    #[serde(with = "color32_serde")]
    pub stop_state: Color32,
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self {
            run_state: Color32::from_rgb(100, 200, 100),
            stop_state: Color32::from_rgb(200, 100, 100),
        }
    }
}

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
            slider_value: 50.0,
            combo_value: "Option A".to_string(),
            time_format: "24h".to_string(),
            colors: LogColors::default(),
            button_colors: ButtonColors::default(),
        }
    }
}
