use chrono::{DateTime, Local};
use eframe::egui;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonColors {
    #[serde(with = "color32_serde")]
    pub run_state: egui::Color32,
    #[serde(with = "color32_serde")]
    pub stop_state: egui::Color32,
}

impl Default for ButtonColors {
    fn default() -> Self {
        Self {
            run_state: egui::Color32::GREEN,    // Green for RUN state
            stop_state: egui::Color32::RED,     // Red for STOP state
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogColors {
    #[serde(with = "color32_serde")]
    pub clock: egui::Color32,
    #[serde(with = "color32_serde")]
    pub slider: egui::Color32,
    #[serde(with = "color32_serde")]
    pub option_a: egui::Color32,
    #[serde(with = "color32_serde")]
    pub option_b: egui::Color32,
    #[serde(with = "color32_serde")]
    pub option_c: egui::Color32,
    #[serde(with = "color32_serde")]
    pub time_format: egui::Color32,
    #[serde(with = "color32_serde")]
    pub custom_event: egui::Color32,
    #[serde(with = "color32_serde")]
    pub run_stop_log: egui::Color32,
}

impl Default for LogColors {
    fn default() -> Self {
        Self {
            clock: egui::Color32::from_rgb(100, 200, 255),     // Light Blue
            slider: egui::Color32::from_rgb(255, 180, 100),    // Orange
            option_a: egui::Color32::from_rgb(255, 150, 150),  // Soft Red
            option_b: egui::Color32::from_rgb(150, 255, 150),  // Soft Green
            option_c: egui::Color32::from_rgb(150, 150, 255),  // Soft Blue
            time_format: egui::Color32::from_rgb(190, 140, 255), // Purple
            custom_event: egui::Color32::from_rgb(255, 215, 0), // Gold
            run_stop_log: egui::Color32::from_rgb(0, 255, 255), // Cyan
        }
    }
}

#[derive(Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub source: String,
    pub message: String,
    pub color: Option<egui::Color32>,
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
