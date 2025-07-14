//! Logger State
//!
//! This module contains the state management for the event logger.
//! It is used by both the UI and the logger backend.

use std::collections::VecDeque;
use egui::RichText;
use crate::components::event_logger::log_colors::LogColors;
use crate::components::event_logger::messages::{LogEntry, Message};
use crate::components::event_logger::log_type::LogType;

/// Maximum number of logs to keep in memory
pub const MAX_LOGS: usize = 1000;

/// Represents the full state of the event logger
#[derive(Clone)]
pub struct LoggerState {
    /// The log entries
    pub logs: VecDeque<LogEntry>,
    /// Color configuration
    pub colors: LogColors,
    /// Whether to show timestamps
    pub show_timestamps: bool,
    /// Whether to show messages
    pub show_messages: bool,
}

impl Default for LoggerState {
    fn default() -> Self {
        Self {
            logs: VecDeque::with_capacity(MAX_LOGS),
            colors: LogColors::default(),
            show_timestamps: true,
            show_messages: true,
        }
    }
}

impl LoggerState {
    /// Create a new logger state with the given colors
    pub fn new(colors: LogColors) -> Self {
        Self {
            logs: VecDeque::with_capacity(MAX_LOGS),
            colors,
            show_timestamps: true,
            show_messages: true,
        }
    }

    /// Add a new log entry
    pub fn add_log(&mut self, entry: LogEntry) {
        self.logs.push_back(entry);
        
        // Maintain circular buffer - remove oldest entry if at capacity
        if self.logs.len() >= MAX_LOGS {
            self.logs.pop_front();
        }
    }
    
    /// Clear all log entries
    pub fn clear(&mut self) {
        self.logs.clear();
    }
    
    /// Update the color scheme
    pub fn update_colors(&mut self, new_colors: LogColors) {
        self.colors = new_colors;
    }
    
    /// Toggle timestamp display
    pub fn toggle_timestamps(&mut self, show: bool) {
        self.show_timestamps = show;
    }
    
    /// Toggle message display
    pub fn toggle_messages(&mut self, show: bool) {
        self.show_messages = show;
    }
    
    /// Export recent log entries
    pub fn export_recent(&self, count: usize) -> Vec<LogEntry> {
        let count = std::cmp::min(count, self.logs.len());
        
        // Get the most recent entries (last 'count' items)
        let start_index = self.logs.len().saturating_sub(count);
        self.logs.iter()
            .skip(start_index)
            .cloned()
            .collect()
    }
    
    /// Process an entry for display, creating formatted rich text
    pub fn format_log_entry(&self, entry: &LogEntry) -> (RichText, RichText) {
        // Format timestamp
        let time_str = entry.timestamp.format("%H:%M:%S%.3f").to_string();
        let time_color = self.colors.time_format;
        let timestamp_rich = RichText::new(time_str).color(time_color);
        
        // Determine message color based on log type
        let msg_color = match entry.style_type {
            LogType::Slider => self.colors.slider,
            LogType::OptionA => self.colors.option_a,
            LogType::OptionB => self.colors.option_b,
            LogType::OptionC => self.colors.option_c,
            LogType::CustomEvent => self.colors.custom_event,
            LogType::Checkbox => self.colors.custom_event,
            LogType::RunStop => self.colors.run_stop_log,
            LogType::Timestamp => self.colors.time_format,
            LogType::Default => egui::Color32::WHITE,
            LogType::Primary => self.colors.clock,
            LogType::Secondary => self.colors.custom_event,
        };
        
        // Get severity color from configuration
        let severity_color = match &entry.message {
            Message::Info(_) => self.colors.info_text,
            Message::Warn(_) => self.colors.warn_text,
            Message::Debug(_) => self.colors.debug_text, 
            Message::Error(_) => self.colors.error_text,
        };
        
        // Format with prefix showing both the message type and sender
        let msg_type = entry.message.type_name();
        let sender_name = entry.sender.display_name();
        let prefix = format!("[{msg_type}] [{sender_name}] ");
        let content = entry.message.content();
        
        // Create formatted message with type prefix and content
        let formatted_msg = format!("{prefix}{content}");
        
        // Use configured color priority
        let final_color = if self.colors.prioritize_style_colors {
            msg_color
        } else {
            severity_color
        };
        let message_rich = RichText::new(&formatted_msg).color(final_color);
        
        (timestamp_rich, message_rich)
    }
}