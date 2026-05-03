use eframe::egui;
use chrono::{DateTime, Local};

/// LoggerPayload
///
/// This struct represents a log entry in the logger.
/// It contains information about the log timestamp, log level, and log message.
///
/// The struct is designed to be used with the ReactiveEventLogger,
/// and provides a fluent API for creating log entries.
#[derive(Clone, Debug)]
pub struct LoggerPayload {
    pub timestamp: TimestampContainer,
    pub log_level: LogLevelContainer,
    pub log_message: MessageContainer,
}

/// TimestampContainer
///
/// Container for timestamp related values
#[derive(Clone, Debug)]
pub struct TimestampContainer {
    pub value: LogValue,
}

/// LogLevelContainer
///
/// Container for different log levels
#[derive(Clone, Debug)]
pub struct LogLevelContainer {
    pub info: LogValue,
    pub debug: LogValue,
    pub warning: LogValue,
    pub error: LogValue,
}

/// MessageContainer
///
/// Container for message content
#[derive(Clone, Debug)]
pub struct MessageContainer {
    pub content: LogValue,
}

/// LogValue
///
/// A value with associated color for display
#[derive(Clone, Debug)]
pub struct LogValue {
    pub value: String,
    pub color: egui::Color32,
}

// Default colors
pub const SOFT_GREEN: egui::Color32 = egui::Color32::from_rgb(150, 255, 150);
pub const SOFT_BLUE: egui::Color32 = egui::Color32::from_rgb(150, 150, 255);
pub const LIGHT_GRAY: egui::Color32 = egui::Color32::from_rgb(180, 180, 180);

impl Default for LoggerPayload {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggerPayload {
    /// Create a new empty log payload
    pub fn new() -> Self {
        Self {
            timestamp: TimestampContainer {
                value: LogValue {
                    value: String::new(),
                    color: LIGHT_GRAY,
                },
            },
            log_level: LogLevelContainer {
                info: LogValue {
                    value: String::new(),
                    color: SOFT_GREEN,
                },
                debug: LogValue {
                    value: String::new(),
                    color: SOFT_BLUE,
                },
                warning: LogValue {
                    value: String::new(),
                    color: egui::Color32::YELLOW,
                },
                error: LogValue {
                    value: String::new(),
                    color: egui::Color32::RED,
                },
            },
            log_message: MessageContainer {
                content: LogValue {
                    value: String::new(),
                    color: egui::Color32::WHITE,
                },
            },
        }
    }
    
    /// Create a new log payload with a custom type
    pub fn with_custom_type(identifier: &str) -> Self {
        let mut payload = Self::new();
        payload.custom_type(identifier);
        payload
    }

    /// Set log level as info
    pub fn info(&mut self) -> &mut Self {
        self.log_level.info.value = "INFO".to_string();
        self.log_level.debug.value = String::new();
        self.log_level.warning.value = String::new();
        self.log_level.error.value = String::new();
        self
    }

    /// Set log level as debug
    pub fn debug(&mut self) -> &mut Self {
        self.log_level.info.value = String::new();
        self.log_level.debug.value = "DEBUG".to_string();
        self.log_level.warning.value = String::new();
        self.log_level.error.value = String::new();
        self
    }

    /// Set log level as warning
    pub fn warning(&mut self) -> &mut Self {
        self.log_level.info.value = String::new();
        self.log_level.debug.value = String::new();
        self.log_level.warning.value = "WARNING".to_string();
        self.log_level.error.value = String::new();
        self
    }

    /// Set log level as error
    pub fn error(&mut self) -> &mut Self {
        self.log_level.info.value = String::new();
        self.log_level.debug.value = String::new();
        self.log_level.warning.value = String::new();
        self.log_level.error.value = "ERROR".to_string();
        self
    }
    
    /// Set a custom log type with the specified identifier
    pub fn custom_type(&mut self, identifier: &str) -> &mut Self {
        // Clear other log levels first
        self.log_level.info.value = String::new();
        self.log_level.debug.value = String::new();
        self.log_level.warning.value = String::new();
        self.log_level.error.value = String::new();
        
        // Store the custom identifier in the info field for now
        // This is for backward compatibility until we refactor the LogLevelContainer
        self.log_level.info.value = format!("CUSTOM:{}", identifier);
        
        self
    }

    /// Set message content
    pub fn message(&mut self, content: String) -> &mut Self {
        self.log_message.content.value = content;
        self
    }

    /// Set all colors at once
    pub fn with_colors(&mut self, timestamp_color: egui::Color32, level_color: egui::Color32, message_color: egui::Color32) -> &mut Self {
        self.with_timestamp_color(timestamp_color)
            .with_level_color(level_color)
            .with_message_color(message_color)
    }

    /// Set timestamp color
    pub fn with_timestamp_color(&mut self, color: egui::Color32) -> &mut Self {
        self.timestamp.value.color = color;
        self
    }

    /// Set level color based on active level
    pub fn with_level_color(&mut self, color: egui::Color32) -> &mut Self {
        if !self.log_level.info.value.is_empty() {
            self.log_level.info.color = color;
        } else if !self.log_level.debug.value.is_empty() {
            self.log_level.debug.color = color;
        } else if !self.log_level.warning.value.is_empty() {
            self.log_level.warning.color = color;
        } else if !self.log_level.error.value.is_empty() {
            self.log_level.error.color = color;
        }
        self
    }

    /// Set message color
    pub fn with_message_color(&mut self, color: egui::Color32) -> &mut Self {
        self.log_message.content.color = color;
        self
    }

    /// Create as message only (no timestamp or level)
    pub fn as_message_only(&mut self) -> &mut Self {
        self.timestamp.value.value = String::new();
        self.log_level.info.value = String::new();
        self.log_level.debug.value = String::new();
        self.log_level.warning.value = String::new();
        self.log_level.error.value = String::new();
        self
    }

    /// Update timestamp to current time and finalize
    pub fn update(&mut self) -> &mut Self {
        // Only add timestamp if it's not already set and this isn't a message-only log
        if self.timestamp.value.value.is_empty() && 
           (!self.log_level.info.value.is_empty() || 
            !self.log_level.debug.value.is_empty() ||
            !self.log_level.warning.value.is_empty() ||
            !self.log_level.error.value.is_empty()) {
            let local: DateTime<Local> = Local::now();
            self.timestamp.value.value = local.format("%Y-%m-%d %H:%M:%S").to_string();
        }
        self
    }
}