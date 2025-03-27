/// State management for the Clock example
/// 
/// This module contains the AppState struct which holds all the reactive state for the Clock example.
/// The AppState struct contains all the reactive values and derived values that are used to manage 
/// the state of the application.
/// 
/// Dynamic<T> is a reactive value that can be updated and read from any part of the application.
/// Derived<T> is a reactive value that is derived from other reactive values. It is recomputed
/// whenever any of the dependent reactive values change.
///
///
use crate::types::{ClockMessage, Config, LogColors, ButtonColors, LogEntry};
use chrono::Local;
use eframe::egui;
use egui_mobius_reactive::{Dynamic, Derived, ReactiveValue};
use std::collections::VecDeque;
use std::sync::Arc;

/// AppState struct holds all the reactive state for the clock_reactive example.
///
/// Dynamic<T> Member of AppState:
///  - slider_value: Dynamic<f32>
///  - combo_value: Dynamic<String>
///  - current_time: Dynamic<String>
///  - logs: Dynamic<VecDeque<LogEntry>>
///  - log_filters: Dynamic<Vec<String>>
///  - buffer_size: Dynamic<usize>
///  - repaint: egui::Context
///  - colors: Dynamic<LogColors>
///  - button_colors: Dynamic<ButtonColors>
///  - button_started: Dynamic<bool>
///  - use_24h: Dynamic<bool>
/// 
/// The logs are what is displayed in the log window. The log_filters are used to filter the logs
/// based on the source of the log. The buffer_size is the maximum number of logs to keep in the
/// logs buffer. The slider_value and combo_value are used to display the current values of the
/// slider and combo box. The current_time is the current time displayed in the clock window.
/// The repaint is used to request a repaint of the UI. The colors are the colors used to display
/// the different types of logs. The button_colors are the colors used to display the run and stop
/// buttons. The button_started is used to keep track of the state of the run/stop button. The use_24h
/// is used to keep track of whether the clock should display time in 24h format or 12h format.
///
/// Derived<T> Member of AppState:
///  - filtered_logs: Derived<VecDeque<LogEntry>>
///  - log_count: Derived<usize>
///  - formatted_time: Derived<String>
/// 
/// The filtered_logs is a derived value that filters the logs based on the log_filters. The log_count
/// is a derived value that keeps track of the number of logs in the logs buffer. The formatted_time
/// is a derived value that formats the current time based on the use_24h value.
///
#[derive(Clone)]
pub struct AppState {
    pub slider_value   : Dynamic<f32>,
    pub combo_value    : Dynamic<String>,
    pub current_time   : Dynamic<String>,
    pub logs           : Dynamic<VecDeque<LogEntry>>,
    pub log_filters    : Dynamic<Vec<String>>,
    pub buffer_size    : Dynamic<usize>,
    pub repaint        : egui::Context,
    pub colors         : Dynamic<LogColors>,
    pub button_colors  : Dynamic<ButtonColors>,
    pub button_started : Dynamic<bool>,
    pub use_24h        : Dynamic<bool>,
    
    // Derived values
    pub filtered_logs  : Derived<VecDeque<LogEntry>>,
    pub log_count      : Derived<usize>,
    pub formatted_time : Derived<String>,
}


impl AppState {
    pub fn new(repaint: egui::Context, config: Config) -> Self {
        // Create base values
        let logs: Dynamic<VecDeque<LogEntry>> = Dynamic::new(VecDeque::with_capacity(1000));
        let logs_clone = logs.clone();
        let logs_clone_2 = logs.clone();
        let logs_clone_3 = logs.clone();
        
        let log_filters: Dynamic<Vec<String>> = Dynamic::new(vec!["ui".to_string(), "clock".to_string()]);
        let log_filters_clone = log_filters.clone();
        let current_time: Dynamic<String> = Dynamic::new(String::new());
        let current_time_clone = current_time.clone();
        let current_time_clone_2 = current_time.clone();
        let use_24h: Dynamic<bool> = Dynamic::new(config.time_format == "24h");
        let use_24h_clone = use_24h.clone();
        let use_24h_clone_2 = use_24h.clone();
        
        // Create derived values
        let deps = [
            Arc::new(logs_clone.clone()) as Arc<dyn ReactiveValue>,
            Arc::new(log_filters_clone.clone()) as Arc<dyn ReactiveValue>
        ];
        let filtered_logs = Derived::new(
            &deps,
            move || {
                let logs = logs_clone_2.get();
                let filters = log_filters_clone.get();
                let mut filtered = VecDeque::new();
                for log in logs.iter() {
                    if filters.contains(&log.source) {
                        filtered.push_back(log.clone());
                    }
                }
                filtered
            }
        );
        
        let deps = [Arc::new(logs_clone_3.clone()) as Arc<dyn ReactiveValue>];
        let log_count = Derived::new(
            &deps,
            move || logs_clone_3.clone().get().len()
        );
        
        let deps = [
            Arc::new(current_time_clone) as Arc<dyn ReactiveValue>,
            Arc::new(use_24h_clone) as Arc<dyn ReactiveValue>
        ];
        let formatted_time = Derived::new(
            &deps,
            move || {
                let time = current_time_clone_2.get();
                let use_24h = use_24h_clone_2.get();
                if use_24h {
                    time.clone()
                } else {
                    time.trim_start_matches('0').to_string()
                }
            }
        );

        Self {
            current_time,
            logs,
            log_filters,
            buffer_size    : Dynamic::new(1000),
            slider_value   : Dynamic::new(config.slider_value),
            combo_value    : Dynamic::new(config.combo_value),
            repaint,
            colors         : Dynamic::new(config.colors),
            button_colors  : Dynamic::new(config.button_colors),
            button_started : Dynamic::new(false),
            use_24h        : Dynamic::new(config.time_format == "24h"),
            filtered_logs,
            log_count,
            formatted_time,
        }
    }

    // handle_message is not really used but is kept here to show 
    // how to handle messages in the AppState struct, particularly if 
    // using the EventTrait from the MobiusReactive crate.
    
    // pub fn handle_message(&self, msg: ClockMessage) {
    //     let now = Local::now();
    //     let use_24h = self.use_24h.get();
    //     let time_str = if use_24h {
    //         now.format("%H:%M:%S").to_string()
    //     } else {
    //         now.format("%I:%M:%S %p").to_string().trim_start_matches('0').to_string()
    //     };
    //     //self.formatted_time.set(time_str);

    //     match msg {
    //         ClockMessage::TimeUpdated(time) => {
    //             // self.current_time.set(time.clone());
    //             // println!("Time updated: {}", time);
    //             // let mut current_logs = self.logs.get();
    //             // if current_logs.len() >= 1000 {
    //             //     current_logs.pop_front();
    //             // }
    //             // current_logs.push_back(LogEntry {
    //             //     timestamp: Local::now(),
    //             //     source: "clock".to_string(),
    //             //     message: format!("Time updated: {}", time),
    //             //     color: Some(egui::Color32::from_rgb(100, 200, 255)), // Light Blue
    //             // });
    //             // self.logs.set(current_logs);
    //         }

    //         ClockMessage::Start => {
    //             self.button_started.set(true);
    //             self.button_colors.set(ButtonColors {
    //                 run_state: egui::Color32::from_rgb(0, 255, 0),
    //                 stop_state: egui::Color32::GRAY,
    //             });
    //         }
    //         ClockMessage::Stop => {
    //             self.button_started.set(false);
    //             self.button_colors.set(ButtonColors {
    //                 run_state: egui::Color32::GRAY,
    //                 stop_state: egui::Color32::from_rgb(255, 0, 0),
    //             });
    //         }
    //         ClockMessage::Clear => {
    //             self.logs.set(VecDeque::new());
    //         }
    //     }
    //     self.repaint.request_repaint();
    // }


    pub fn save_config(&self) {
        let config = Config {
            slider_value  : self.slider_value.get(),
            combo_value   : self.combo_value.get(),
            time_format   : if self.use_24h.get() { "24h" } else { "12h" }.to_string(),
            colors        : self.colors.get(),
            button_colors : self.button_colors.get(),
        };

        // Move config saving to background thread to avoid blocking UI
        let config_json = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write("config.json", config_json).ok();
    }

    pub fn log(&self, source: &str, message: String) {
        let colors = self.colors.get();
        let color = if source == "clock" {
            Some(colors.clock)
        } else if source == "ui" {
            if message.contains("Slider value") {
                Some(colors.slider)
            } else if message.contains("Selected option: Option A") {
                Some(colors.option_a)
            } else if message.contains("Selected option: Option B") {
                Some(colors.option_b)
            } else if message.contains("Selected option: Option C") {
                Some(colors.option_c)
            } else if message.contains("Process") {
                Some(colors.run_stop_log)
            } else {
                Some(colors.custom_event)  // Default for UI events and Custom Events
            }
        } else {
            None
        };

        let entry = LogEntry {
            timestamp: Local::now(),
            source: source.to_string(),
            message,
            color,
        };

        let _ = entry.save_to_file();

        let mut logs = self.logs.get();
        logs.push_back(entry);
        let max_size = self.buffer_size.get();
        if logs.len() > max_size {
            let len = logs.len();
            logs.drain(0..len - max_size);
        }
        self.logs.set(logs);
    }
}
