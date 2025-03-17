use crate::logger::{LogColors, LogEntry};
use crate::types::{ClockMessage, Event, Config};
use chrono::Local;
use eframe::egui;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;
use egui_mobius::types::Value;

pub struct AppState {
    pub slider_value: Value<f32>,
    pub combo_value: Value<String>,
    pub current_time: Value<String>,
    pub logs: Value<Vec<LogEntry>>,
    pub log_filters: Value<Vec<String>>,
    pub repaint: egui::Context,
    pub event_signal: Value<Option<Signal<Event>>>,
    pub colors: Value<LogColors>,
    pub use_24h: Value<bool>,
}

impl AppState {
    pub fn new(repaint: egui::Context, config: Config) -> Self {
        Self {
            current_time: Value::new(String::new()),
            logs: Value::new(Vec::new()),
            log_filters: Value::new(vec!["ui".to_string(), "clock".to_string()]),
            slider_value: Value::new(config.slider_value),
            combo_value: Value::new(config.combo_value),
            repaint,
            event_signal: Value::new(None),
            colors: Value::new(config.colors),
            use_24h: Value::new(config.time_format == "24h"),
        }
    }

    pub fn set_event_signal(&self, signal: Signal<Event>) {
        *self.event_signal.lock().unwrap() = Some(signal);
    }

    pub fn set_clock_slot(&self, mut slot: Slot<ClockMessage>) {
        let ctx = self.repaint.clone();
        let current_time = self.current_time.clone();
        let logs = self.logs.clone();
        let use_24h = self.use_24h.clone();

        slot.start(move |msg| {
            let ClockMessage::TimeUpdated(_) = msg;
            let now = Local::now();
            let time_str = if *use_24h.lock().unwrap() {
                now.format("%H:%M:%S").to_string()
            } else {
                now.format("%I:%M:%S %p").to_string().trim_start_matches('0').to_string()
            };
            *current_time.lock().unwrap() = time_str.clone();
            let mut logs = logs.lock().unwrap();
            logs.push(LogEntry {
                timestamp: now,
                source: "clock".to_string(),
                message: format!("Time updated: {}", time_str),
                color: Some(egui::Color32::from_rgb(100, 200, 255)), // Light Blue
            });
            ctx.request_repaint();
        });
    }

    pub fn save_config(&self) {
        let config = Config {
            slider_value: *self.slider_value.lock().unwrap(),
            combo_value: self.combo_value.lock().unwrap().clone(),
            time_format: if *self.use_24h.lock().unwrap() { "24h" } else { "12h" }.to_string(),
            colors: self.colors.lock().unwrap().clone(),
        };
        if let Ok(json_data) = serde_json::to_string_pretty(&config) {
            let local_dir = std::path::Path::new(".local");
            if !local_dir.exists() {
                let _ = std::fs::create_dir_all(local_dir);
            }
            let config_path = local_dir.join("config.json");
            if let Err(e) = std::fs::write(&config_path, json_data) {
                eprintln!("Failed to save config: {}", e);
            }
        }
    }

    pub fn log(&self, source: &str, message: String) {
        let entry = LogEntry {
            timestamp: Local::now(),
            source: source.to_string(),
            message,
            color: None,
        };

        let _ = entry.save_to_file();

        let mut logs = self.logs.lock().unwrap();
        logs.push(entry);
        if logs.len() > 1000 {
            let len = logs.len();
            logs.drain(0..len - 1000);
        }
    }
}
