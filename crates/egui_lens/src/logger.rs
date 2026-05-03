//! Reactive Event Logger - a stateful logger responding to events. 
//! This module provides a reactive event logger for the Mobius framework.
//! It allows for logging messages with different severity levels and filtering them.
//! The logger supports custom log types and provides a user interface for filtering and displaying logs.
//!
//! The logger is designed to be used with the Mobius framework and is reactive to changes in the state.
//! It uses the `egui` library for the user interface and `egui_mobius_reactive` for reactivity.
//!
//! The logger supports the following log levels:
//! - INFO
//! - WARNING
//! - ERROR
//! - DEBUG
//! - CUSTOM (for custom log types)
//!
//! The logger also supports filtering logs by type and by text content.
//!
//! The filtering options are stored in a `LogFilter` struct, which can be modified by the user.
//! The logger state is stored in a `ReactiveEventLoggerState` struct, which is shared across the application.
//!
use eframe::egui;
use egui_mobius_reactive::{Dynamic, ReactiveWidgetRef};
use crate::payload::LoggerPayload;
use crate::logger_colors::LogColors;

/// LogType
///
/// This enum is used to categorize the type of log entry
/// in the terminal widget. This is used to color code the
/// log entries.
///
/// Standard log levels (INFO, WARNING, ERROR, DEBUG) are provided
/// along with additional generic types for various use cases.
/// 
/// The Custom variant accepts a String parameter, allowing
/// for extensible custom log types with arbitrary identifiers.
#[derive(Clone, PartialEq)]
pub enum LogType {
    /// Standard info level messages
    Info,
    /// Warning messages
    Warning,
    /// Error messages
    Error,
    /// Debug messages
    Debug,
    /// For timestamps or time-related information
    Timestamp,
    /// For system events
    System,
    /// For user interactions
    UserAction,
    /// For configuration changes
    Config,
    /// For general status updates
    Status,
    /// For progress indicators
    Progress,
    /// For success messages
    Success,
    /// For neutral/default messages
    Default,
    /// For custom types with a specific identifier string
    Custom(String),
}

/// LogFilter
///
/// Encapsulates filtering options for log messages.
/// This struct controls which log types are displayed and provides
/// text-based filtering capabilities.
#[derive(Clone, Debug)]
pub struct LogFilter {
    /// Show/hide INFO logs
    pub show_info: bool,
    /// Show/hide WARNING logs
    pub show_warning: bool,
    /// Show/hide ERROR logs
    pub show_error: bool,
    /// Show/hide DEBUG logs
    pub show_debug: bool,
    /// Show/hide custom log types
    pub show_custom: bool,
    /// Show/hide system logs
    pub show_system: bool,
    /// Text filter to search in log messages (case-insensitive)
    pub text_filter: String,
}

impl Default for LogFilter {
    fn default() -> Self {
        Self {
            show_info: true,
            show_warning: true,
            show_error: true,
            show_debug: true,
            show_custom: true,
            show_system: true,
            text_filter: String::new(),
        }
    }
}

impl LogFilter {
    /// Create a new LogFilter with default settings (show all)
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Check if a log should be displayed based on current filter settings
    pub fn should_display(&self, log: &LoggerPayload) -> bool {
        // First check log type filtering
        let passes_type_filter = if !log.log_level.info.value.is_empty() {
            // Check if it's a custom type
            if log.log_level.info.value.starts_with("CUSTOM:") {
                self.show_custom
            } else {
                self.show_info
            }
        } else if !log.log_level.warning.value.is_empty() {
            self.show_warning
        } else if !log.log_level.error.value.is_empty() {
            self.show_error
        } else if !log.log_level.debug.value.is_empty() {
            self.show_debug
        } else {
            // For other system messages or messages without explicit level
            self.show_system
        };
        
        // If it doesn't pass the type filter, no need to check text filter
        if !passes_type_filter {
            return false;
        }
        
        // If text filter is empty, all logs pass the text filter
        if self.text_filter.is_empty() {
            return true;
        }
        
        // Check if the message contains the text filter (case-insensitive)
        let lowercase_message = log.log_message.content.value.to_lowercase();
        let lowercase_filter = self.text_filter.to_lowercase();
        
        lowercase_message.contains(&lowercase_filter)
    }
    
    /// Reset all filters to default (show all)
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    
    /// Save filter state to memory for persistence between sessions
    pub fn save_to_memory(&self, ctx: &egui::Context) {
        ctx.memory_mut(|mem| {
            // Store each filter setting as a separate value for better persistence
            mem.data.insert_persisted(egui::Id::new("logger_filter_show_info"), self.show_info);
            mem.data.insert_persisted(egui::Id::new("logger_filter_show_warning"), self.show_warning);
            mem.data.insert_persisted(egui::Id::new("logger_filter_show_error"), self.show_error);
            mem.data.insert_persisted(egui::Id::new("logger_filter_show_debug"), self.show_debug);
            mem.data.insert_persisted(egui::Id::new("logger_filter_show_custom"), self.show_custom);
            mem.data.insert_persisted(egui::Id::new("logger_filter_show_system"), self.show_system);
            mem.data.insert_persisted(egui::Id::new("logger_filter_text"), self.text_filter.clone());
        });
    }
    
    /// Load filter state from memory
    pub fn load_from_memory(&mut self, ctx: &egui::Context) {
        // Use temporary variables to store the values from memory
        let show_info = ctx.memory_mut(|mem| mem.data.get_persisted::<bool>(egui::Id::new("logger_filter_show_info")));
        let show_warning = ctx.memory_mut(|mem| mem.data.get_persisted::<bool>(egui::Id::new("logger_filter_show_warning")));
        let show_error = ctx.memory_mut(|mem| mem.data.get_persisted::<bool>(egui::Id::new("logger_filter_show_error")));
        let show_debug = ctx.memory_mut(|mem| mem.data.get_persisted::<bool>(egui::Id::new("logger_filter_show_debug")));
        let show_custom = ctx.memory_mut(|mem| mem.data.get_persisted::<bool>(egui::Id::new("logger_filter_show_custom")));
        let show_system = ctx.memory_mut(|mem| mem.data.get_persisted::<bool>(egui::Id::new("logger_filter_show_system")));
        let text_filter = ctx.memory_mut(|mem| mem.data.get_persisted::<String>(egui::Id::new("logger_filter_text")));
        
        // Apply the values if they were found
        if let Some(value) = show_info {
            self.show_info = value;
        }
        if let Some(value) = show_warning {
            self.show_warning = value;
        }
        if let Some(value) = show_error {
            self.show_error = value;
        }
        if let Some(value) = show_debug {
            self.show_debug = value;
        }
        if let Some(value) = show_custom {
            self.show_custom = value;
        }
        if let Some(value) = show_system {
            self.show_system = value;
        }
        if let Some(value) = text_filter {
            self.text_filter = value;
        }
    }
}

/// Debug for LogType
/// 
/// This is used to display the LogType in the terminal widget
/// 
impl std::fmt::Debug for LogType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogType::Info => write!(f, "INFO"),
            LogType::Warning => write!(f, "WARNING"),
            LogType::Error => write!(f, "ERROR"),
            LogType::Debug => write!(f, "DEBUG"),
            LogType::Timestamp => write!(f, "TIME"),
            LogType::System => write!(f, "SYSTEM"),
            LogType::UserAction => write!(f, "USER"),
            LogType::Config => write!(f, "CONFIG"),
            LogType::Status => write!(f, "STATUS"),
            LogType::Progress => write!(f, "PROGRESS"),
            LogType::Success => write!(f, "SUCCESS"),
            LogType::Default => write!(f, "DEFAULT"),
            LogType::Custom(identifier) => write!(f, "CUSTOM:{}", identifier),
        }
    }
}


// This constant is now directly used in ReactiveEventLoggerState::new()

/// ReactiveEventLoggerState
/// 
/// This struct handles the state of the event logger panel.
/// It is used to store the state of the logger panel, and to determine
/// which columns to show in the logger panel. The state is stored
/// in a shared state, which is used to update the logger panel
/// when the state changes.
/// 
/// It maintains a circular buffer of log messages with a maximum capacity
/// of 1000 entries. When the buffer is full, the oldest entry is removed
/// before adding a new one.
#[derive(Default, Clone)]
pub struct ReactiveEventLoggerState {
    pub show_timestamps : bool,               // show/hide timestamps
    pub show_log_level  : bool,               // show/hide log level
    pub show_messages   : bool,               // show/hide messages
    pub logs            : Vec<LoggerPayload>, // store log messages in a circular buffer
    pub max_logs        : usize,              // maximum number of log entries to store
    pub filter          : LogFilter,          // filtering options for log messages
}

impl ReactiveEventLoggerState {
    pub fn new() -> Self {
        // Maximum number of logs to keep is hardcoded to 1000
        const MAX_LOGS: usize = 1000;
        
        Self {
            show_timestamps : true,
            show_log_level  : true,
            show_messages   : true,
            logs            : Vec::with_capacity(MAX_LOGS),
            max_logs        : MAX_LOGS,
            filter          : LogFilter::default(),
        }
    }

    /// Add a log entry to the circular buffer
    /// If the buffer is full, the oldest entry is removed
    pub fn add_log(&mut self, log: LoggerPayload) {
        // If we've reached capacity, remove the oldest entry (front of the vector)
        if self.logs.len() >= self.max_logs {
            self.logs.remove(0); // Remove the first (oldest) element
        }
        
        // Add the new log entry at the end
        self.logs.push(log);
    }
    
    /// Clear all log entries
    pub fn clear_logs(&mut self) {
        self.logs.clear();
    }
    
    /// Get the number of log entries
    pub fn log_count(&self) -> usize {
        self.logs.len()
    }
    
    /// Set the maximum number of log entries
    #[allow(dead_code)]
    pub fn set_max_logs(&mut self, max_logs: usize) {
        self.max_logs = max_logs;
        
        // If the current number of logs exceeds the new maximum,
        // remove the oldest entries until we're at the new maximum
        while self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
    }
}

/// ReactiveEventLogger
/// 
/// This struct is the main component for logging events in the application.
/// It processes LoggerPayload objects which contain:
///
/// - Timestamp - When the log was created
/// - LogLevel - The severity level (info, warn, debug, error)
/// - Message - The actual content
///
/// The logger provides a terminal-like interface that displays logs in a table format.
/// Users can toggle which columns to display (timestamps, log levels, messages).
///
/// The ReactiveEventLogger uses a shared state (ReactiveEventLoggerState) to maintain 
/// the log entries and display settings, making it reactive to UI changes.
pub struct ReactiveEventLogger<'a> {
    state: &'a Dynamic<ReactiveEventLoggerState>,  // shared state of the logger panel
    colors: Option<&'a Dynamic<LogColors>>,        // optional colors for the log messages
}

impl<'a> ReactiveEventLogger<'a> {
    /// Create a new ReactiveEventLogger with a shared state
    #[allow(dead_code)]
    pub fn new(state: &'a Dynamic<ReactiveEventLoggerState>) -> Self {
        Self {
            state,
            colors: None,
        }
    }
    
    /// Save colors to gerber_viewer specific config directory
    fn save_colors_for_gerber_viewer(colors: &LogColors) {
        use std::path::PathBuf;
        use std::fs;
        
        let colors = colors.clone();
        std::thread::spawn(move || {
            // Get config directory path for gerber_viewer
            let config_dir = dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("gerber_viewer");
            
            // Create config directory if it doesn't exist
            if let Err(e) = fs::create_dir_all(&config_dir) {
                eprintln!("Failed to create config directory: {}", e);
                return;
            }
            
            // Create config file path
            let config_path = config_dir.join("log_colors.json");
            
            // Serialize colors to JSON
            match serde_json::to_string_pretty(&colors) {
                Ok(json) => {
                    // Write JSON to file
                    if let Err(e) = fs::write(&config_path, json) {
                        eprintln!("Failed to write colors to {}: {}", config_path.display(), e);
                    } else {
                        println!("Successfully saved colors to {}", config_path.display());
                    }
                },
                Err(e) => eprintln!("Failed to serialize colors: {}", e),
            }
        });
    }
    
    /// Show the filter modal dialog
    fn show_filter_modal(&self, ui: &mut egui::Ui) {
        // Check if filter modal should be shown
        let show_filter_modal = ui.ctx().memory(|mem| {
            mem.data.get_temp::<bool>(egui::Id::new("show_logger_filter_modal")).unwrap_or(false)
        });
        
        if show_filter_modal {
            // Get reference to the state
            let state_ref = ReactiveWidgetRef::from_dynamic(self.state);
            
            if let Some(state_arc) = state_ref.weak_ref.upgrade() {
                // Get a mutable reference to the state
                if let Ok(mut state) = state_arc.lock() {
                    // Create a local copy of the filter for editing
                    let mut filter = state.filter.clone();
                    
                    // Load saved filter settings (only once when opening the modal)
                    filter.load_from_memory(ui.ctx());
                    
                    // Create modal window
                    let modal_id = egui::Id::new("logger_filter_modal");
                    egui::Window::new("Log Filters")
                        .id(modal_id)
                        .default_size(egui::Vec2::new(300.0, 350.0))  
                        .min_size(egui::Vec2::new(250.0, 250.0))      
                        .collapsible(false)
                        .resizable(true)
                        .title_bar(true)
                        .show(ui.ctx(), |ui| {
                            let mut changed = false;
                            
                            ui.vertical(|ui| {
                                ui.heading("Log Types");
                                ui.add_space(8.0);
                                
                                // Log level filters
                                ui.horizontal(|ui| {
                                    if ui.checkbox(&mut filter.show_info, "INFO").changed() {
                                        changed = true;
                                    }
                                    
                                    if ui.checkbox(&mut filter.show_warning, "WARNING").changed() {
                                        changed = true;
                                    }
                                });
                                
                                ui.horizontal(|ui| {
                                    if ui.checkbox(&mut filter.show_error, "ERROR").changed() {
                                        changed = true;
                                    }
                                    
                                    if ui.checkbox(&mut filter.show_debug, "DEBUG").changed() {
                                        changed = true;
                                    }
                                });
                                
                                ui.horizontal(|ui| {
                                    if ui.checkbox(&mut filter.show_custom, "CUSTOM").changed() {
                                        changed = true;
                                    }
                                    
                                    if ui.checkbox(&mut filter.show_system, "SYSTEM").changed() {
                                        changed = true;
                                    }
                                });
                                
                                ui.add_space(16.0);
                                
                                // Text filter
                                ui.heading("Text Filter");
                                ui.add_space(4.0);
                                
                                ui.horizontal(|ui| {
                                    ui.label("Contains:");
                                    if ui.text_edit_singleline(&mut filter.text_filter).changed() {
                                        changed = true;
                                    }
                                });
                                
                                ui.label("Case-insensitive search in log messages");
                                
                                ui.add_space(16.0);
                                
                                // Actions
                                ui.horizontal(|ui| {
                                    if ui.button("Reset All").clicked() {
                                        filter.reset();
                                        changed = true;
                                    }
                                    
                                    // Spacer to push the Close button to the right
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button("Close").clicked() {
                                            // Close the modal
                                            ui.ctx().memory_mut(|mem| {
                                                mem.data.remove::<bool>(egui::Id::new("show_logger_filter_modal"));
                                            });
                                        }
                                    });
                                });
                            });
                            
                            // Apply changes if filter was modified
                            if changed {
                                state.filter = filter.clone();
                                
                                // Save filter settings for persistence
                                filter.save_to_memory(ui.ctx());
                            }
                        });
                }
            }
        }
    }
    
    /// Create a new ReactiveEventLogger with a shared state and colors
    pub fn with_colors(state: &'a Dynamic<ReactiveEventLoggerState>, colors: &'a Dynamic<LogColors>) -> Self {
        Self {
            state,
            colors: Some(colors),
        }
    }
    
    #[allow(dead_code)]
    /// Create a new ReactiveEventLogger with the original Dynamic reference
    /// Use this method when you have a ReactiveWidgetRef and want to create a logger
    pub fn from_widget_ref(state: &'a Dynamic<ReactiveEventLoggerState>) -> Self {
        Self {
            state,
            colors: None,
        }
    }
    
    /// Add a log entry from a message string with a specific log level
    pub fn add_log(&self, level: &str, message: &str) {
        let mut payload = LoggerPayload::new();
    
        // Check if it's a custom type (starts with "custom:")
        if level.to_lowercase().starts_with("custom:") {
            // Extract the custom identifier (everything after "custom:")
            let identifier = level[7..].trim();
            
            // Get colors if available
            if let Some(colors_dynamic) = self.colors {
                let colors = colors_dynamic.get();
                
                // Get custom colors for this identifier
                let level_color = colors.get_custom_color_level(identifier);
                let message_color = colors.get_custom_color_message(identifier);
                
                payload.custom_type(identifier)
                       .with_timestamp_color(egui::Color32::from_rgb(180, 180, 180))
                       .with_level_color(level_color)
                       .with_message_color(message_color);
            } else {
                // Fallback if no colors are available
                payload.custom_type(identifier)
                       .with_timestamp_color(egui::Color32::from_rgb(180, 180, 180))
                       .with_level_color(egui::Color32::from_rgb(220, 220, 220)) // Default light gray
                       .with_message_color(egui::Color32::from_rgb(220, 220, 220)); // Default light gray
            }
        } else {
            match level.to_lowercase().as_str() {
                "info" => {
                    // Check if we have custom colors
                    if let Some(colors_dynamic) = self.colors {
                        let colors = colors_dynamic.get();
                        payload.info()
                               .with_timestamp_color(colors.timestamp)
                               .with_level_color(colors.info_level)
                               .with_message_color(colors.info_message);
                    } else {
                        // Default colors if no custom colors available
                        payload.info()
                               .with_timestamp_color(egui::Color32::from_rgb(180, 180, 180))
                               .with_level_color(egui::Color32::from_rgb(150, 255, 150))
                               .with_message_color(egui::Color32::from_rgb(180, 255, 180));
                    }
                },
                "warn" | "warning" => {
                    if let Some(colors_dynamic) = self.colors {
                        let colors = colors_dynamic.get();
                        payload.warning()
                               .with_timestamp_color(colors.timestamp)
                               .with_level_color(colors.warning_level)
                               .with_message_color(colors.warning_message);
                    } else {
                        payload.warning()
                               .with_timestamp_color(egui::Color32::from_rgb(180, 180, 180))
                               .with_level_color(egui::Color32::from_rgb(255, 255, 100))
                               .with_message_color(egui::Color32::from_rgb(255, 255, 140));
                    }
                },
                "debug" => {
                    if let Some(colors_dynamic) = self.colors {
                        let colors = colors_dynamic.get();
                        payload.debug()
                               .with_timestamp_color(colors.timestamp)
                               .with_level_color(colors.debug_level)
                               .with_message_color(colors.debug_message);
                    } else {
                        payload.debug()
                               .with_timestamp_color(egui::Color32::from_rgb(180, 180, 180))
                               .with_level_color(egui::Color32::from_rgb(150, 150, 255))
                               .with_message_color(egui::Color32::from_rgb(180, 180, 255));
                    }
                },
                "error" => {
                    if let Some(colors_dynamic) = self.colors {
                        let colors = colors_dynamic.get();
                        payload.error()
                               .with_timestamp_color(colors.timestamp)
                               .with_level_color(colors.error_level)
                               .with_message_color(colors.error_message);
                    } else {
                        payload.error()
                               .with_timestamp_color(egui::Color32::from_rgb(180, 180, 180))
                               .with_level_color(egui::Color32::from_rgb(255, 100, 100))
                               .with_message_color(egui::Color32::from_rgb(255, 140, 140));
                    }
                },
                _ => {
                    payload.info();
                }
            }
        }
    
        payload.message(message.to_string())
               .update();
    
        self.process_log(&payload);
    }
    
    
    #[allow(dead_code)]
    /// Clear all logs
    pub fn clear(&self) {
        if let Some(state_arc) = ReactiveWidgetRef::from_dynamic(self.state).weak_ref.upgrade() {
            let mut state = state_arc.lock().unwrap();
            state.clear_logs();
        }
    }

    /// Processes a new log entry and adds it to the shared state
    /// Process a log payload and add it to the logger state
    pub fn process_log(&self, log: &LoggerPayload) {
        if let Some(state_arc) = ReactiveWidgetRef::from_dynamic(self.state).weak_ref.upgrade() {
            let mut state = state_arc.lock().unwrap();
            // Only add non-empty logs
            if !log.timestamp.value.value.is_empty() {
                state.add_log(log.clone());
            }
        }
    }
    
    #[allow(dead_code)]
    /// Create and add a simple message-only log with the given content
    pub fn log_message(&self, content: &str) {
        let mut message = LoggerPayload::new();
        message.as_message_only()
               .message(content.to_string())
               .update();
            
        self.process_log(&message);
    }
    
    /// Create and add an info level log with the given content
    pub fn log_info(&self, content: &str) {
        self.add_log("info", content);
    }
    
    /// Create and add a warning level log with the given content
    pub fn log_warning(&self, content: &str) {
        self.add_log("warning", content);
    }
    
    /// Create and add a debug level log with the given content
    pub fn log_debug(&self, content: &str) {
        self.add_log("debug", content);
    }
    
    /// Create and add an error level log with the given content
    pub fn log_error(&self, content: &str) {
        self.add_log("error", content);
    }
    
    /// Create and add a custom log with the given type identifier and content
    pub fn log_custom(&self, custom_type: &str, content: &str) {
        self.add_log(&format!("custom:{}", custom_type), content);
    }

    /// Format logs for export
    fn format_logs_for_export(&self, state: &ReactiveEventLoggerState) -> String {
        let mut log_content = String::new();
        
        // Add a header with timestamp
        log_content.push_str("--- Logger Export ---\n");
        log_content.push_str(&format!("Exported: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // Process logs chronologically (oldest first)
        for log in state.logs.iter() {
            let mut line = String::new();
            
            // Add timestamp if available
            if !log.timestamp.value.value.is_empty() {
                line.push_str(&format!("[{}] ", log.timestamp.value.value));
            }
            
            // Add log level if available
            if !log.log_level.info.value.is_empty() {
                line.push_str(&format!("[{}] ", log.log_level.info.value));
            } else if !log.log_level.debug.value.is_empty() {
                line.push_str(&format!("[{}] ", log.log_level.debug.value));
            } else if !log.log_level.warning.value.is_empty() {
                line.push_str(&format!("[{}] ", log.log_level.warning.value));
            } else if !log.log_level.error.value.is_empty() {
                line.push_str(&format!("[{}] ", log.log_level.error.value));
            }
            
            // Add message
            line.push_str(&log.log_message.content.value);
            line.push('\n');
            
            log_content.push_str(&line);
        }
        
        log_content
    }
    
    /// Save logs to a file
    #[allow(dead_code)]
    fn save_logs_to_file(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        if let Some(state_arc) = ReactiveWidgetRef::from_dynamic(self.state).weak_ref.upgrade() {
            if let Ok(state) = state_arc.lock() {
                let log_content = self.format_logs_for_export(&state);
                std::fs::write(path, log_content)?;
                return Ok(());
            }
        }
        
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Failed to access log data"))
    }

    /// Display the logger UI
    pub fn show(&self, ui: &mut egui::Ui) {
        // Get a reference to the state
        let mut state_ref = ReactiveWidgetRef::from_dynamic(self.state);
        
        // Make sure we have the latest state
        if state_ref.cached_value.is_none() {
            if let Some(arc) = state_ref.weak_ref.upgrade() {
                let guard = arc.lock().unwrap();
                state_ref.cached_value = Some((*guard).clone());
            } else {
                // If we can't get the state, show a placeholder
                ui.label("Logger state unavailable");
                return;
            }
        }
        
        // Get a reference to the cached state
        let state_value = match &state_ref.cached_value {
            Some(value) => value,
            None => {
                ui.label("Logger state unavailable");
                return;
            }
        };
        
        ui.vertical(|ui| {
            // Top row with buffer status and clear button
            ui.horizontal(|ui| {
                // Show buffer status
                ui.label(format!("Logs: {}/{}", state_value.log_count(), state_value.max_logs));
                
                // Add spacing to push buttons to the right
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // Clear logs button
                    if ui.button("Clear Logs").clicked() {
                        // Clear logs if button clicked
                        if let Some(arc) = state_ref.weak_ref.upgrade() {
                            let mut state = arc.lock().unwrap();
                            state.clear_logs();
                        }
                    }
                    
                    // Add small spacing between buttons
                    ui.add_space(8.0);
                    
                    // Add Save Logs button
                    if ui.button("💾 Save Logs").clicked() {
                        // Set a flag to open the save dialog
                        ui.ctx().memory_mut(|mem| {
                            mem.data.insert_temp(egui::Id::new("show_save_logs_dialog"), true);
                        });
                    }
                    
                    // Add small spacing between buttons
                    ui.add_space(8.0);
                    
                    // Add Logger Colors button
                    if ui.button("🎨 Logger Colors").clicked() {
                        // Set a flag to open the color dialog
                        ui.ctx().memory_mut(|mem| {
                            mem.data.insert_temp(egui::Id::new("show_logger_colors_modal"), true);
                        });
                    }
                    
                    // Add small spacing between buttons
                    ui.add_space(8.0);
                    
                    // Add Filter button
                    let filter_button_text = if is_any_filter_active(&state_value.filter) {
                        "🔍 Filters (Active)"
                    } else {
                        "🔍 Filters"
                    };
                    
                    if ui.button(filter_button_text).clicked() {
                        // Signal to open the filter modal
                        ui.ctx().memory_mut(|mem| {
                            mem.data.insert_temp(egui::Id::new("show_logger_filter_modal"), true);
                        });
                    }
                    
                    // Add small spacing between buttons
                    ui.add_space(8.0);
                    
                    // Add System Info button (we keep this in the UI, but application should implement it)
                    if ui.button("📊 System Info").clicked() {
                        // Signal to the application to show system info
                        ui.ctx().memory_mut(|mem| {
                            mem.data.insert_temp(egui::Id::new("show_system_info"), true);
                        });
                    }
                });
            });
            
            ui.separator();
            
            // Display column visibility controls
            ui.horizontal(|ui| {
                ui.label("Display columns:");
                
                // Create local copies of visibility flags
                let mut show_timestamps = state_value.show_timestamps;
                let mut show_log_level = state_value.show_log_level;
                let mut show_messages = state_value.show_messages;
                
                // Timestamps checkbox
                if ui.checkbox(&mut show_timestamps, "Timestamps").changed() {
                    // Update the shared state if changed
                    if let Some(arc) = state_ref.weak_ref.upgrade() {
                        if let Ok(mut state) = arc.lock() {
                            state.show_timestamps = show_timestamps;
                        }
                    }
                }
                
                // Log Level checkbox
                if ui.checkbox(&mut show_log_level, "Log Level").changed() {
                    // Update the shared state if changed
                    if let Some(arc) = state_ref.weak_ref.upgrade() {
                        let mut state = arc.lock().unwrap();
                        state.show_log_level = show_log_level;
                    }
                }

                // Messages checkbox
                if ui.checkbox(&mut show_messages, "Messages").changed() {
                    // Update the shared state if changed
                    if let Some(arc) = state_ref.weak_ref.upgrade() {
                        let mut state = arc.lock().unwrap();
                        state.show_messages = show_messages;
                    }
                }
            });
            
            // Display terminal content using the cached state value
            self.show_event_log_content(ui, state_value);
            
            // Show color picker modal if needed
            self.show_color_picker_modal(ui);
            
            // Show filter modal if needed
            self.show_filter_modal(ui);
            
            // Show save dialog if needed
            self.show_save_dialog(ui);
        });
    }

    /// Show file save dialog
    fn show_save_dialog(&self, ui: &mut egui::Ui) {
        // Check if save dialog should be shown
        let show_save_dialog = ui.ctx().memory(|mem| {
            mem.data.get_temp::<bool>(egui::Id::new("show_save_logs_dialog")).unwrap_or(false)
        });
        
        if show_save_dialog {
            // Use a future to handle the async file dialog
            ui.ctx().memory_mut(|mem| {
                // Clear the flag first to prevent duplicate dialogs
                mem.data.remove::<bool>(egui::Id::new("show_save_logs_dialog"));
                
                // Create a new thread to show the file dialog
                // This avoids blocking the UI thread
                let ctx = ui.ctx().clone();
                let state_clone = self.state.clone();
                std::thread::spawn(move || {
                    // Show native file dialog
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("Text files", &["txt"])
                        .add_filter("Log files", &["log"])
                        .add_filter("All files", &["*"])
                        .set_file_name("logs.txt")
                        .set_title("Save Log File")
                        .save_file() {
                        
                        // Try to save the file
                        if let Some(state_arc) = ReactiveWidgetRef::from_dynamic(&state_clone).weak_ref.upgrade() {
                            if let Ok(state) = state_arc.lock() {
                                let reactive_logger = ReactiveEventLogger::new(&state_clone);
                                let log_content = reactive_logger.format_logs_for_export(&state);
                                
                                // Save the logs to the file
                                if let Err(err) = std::fs::write(&path, log_content) {
                                    // On error, set a flag to show an error message
                                    ctx.memory_mut(|mem| {
                                        mem.data.insert_temp(egui::Id::new("save_logs_error"), 
                                            format!("Failed to save logs: {}", err));
                                    });
                                } else {
                                    // On success, set a flag to show a success message
                                    ctx.memory_mut(|mem| {
                                        mem.data.insert_temp(egui::Id::new("save_logs_success"), 
                                            format!("Logs saved to: {}", path.display()));
                                    });
                                }
                            }
                        }
                    }
                    
                    // Request a repaint to show any success/error messages
                    ctx.request_repaint();
                });
            });
        }
        
        // Show success message if present
        if let Some(success_msg) = ui.ctx().memory(|mem| {
            mem.data.get_temp::<String>(egui::Id::new("save_logs_success"))
        }) {
            // Create a temporary success notification
            let toast = egui::Window::new("✓ Success")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::RIGHT_BOTTOM, [10.0, 10.0])
                .fixed_size([300.0, 80.0]);
                
            toast.show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(5.0);
                    ui.label(success_msg);
                    ui.add_space(5.0);
                    
                    if ui.button("Close").clicked() {
                        ui.ctx().memory_mut(|mem| {
                            mem.data.remove::<String>(egui::Id::new("save_logs_success"));
                        });
                    }
                });
            });
            
            // Automatically clear after 5 seconds
            ui.ctx().request_repaint_after(std::time::Duration::from_secs(5));
        }
        
        // Show error message if present
        if let Some(error_msg) = ui.ctx().memory(|mem| {
            mem.data.get_temp::<String>(egui::Id::new("save_logs_error"))
        }) {
            // Create a temporary error notification
            let toast = egui::Window::new("❌ Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::RIGHT_BOTTOM, [10.0, 10.0])
                .fixed_size([300.0, 80.0]);
                
            toast.show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(5.0);
                    ui.label(error_msg);
                    ui.add_space(5.0);
                    
                    if ui.button("Close").clicked() {
                        ui.ctx().memory_mut(|mem| {
                            mem.data.remove::<String>(egui::Id::new("save_logs_error"));
                        });
                    }
                });
            });
            
            // Automatically clear after 5 seconds
            ui.ctx().request_repaint_after(std::time::Duration::from_secs(5));
        }
    }
    
    /// Display a modal dialog with color pickers for log components
    fn show_color_picker_modal(&self, ui: &mut egui::Ui) {
        // Only show if we have colors available
        if let Some(colors_dynamic) = self.colors {
            // Check if modal should be shown
            let show_modal = ui.ctx().memory(|mem| {
                mem.data.get_temp::<bool>(egui::Id::new("show_logger_colors_modal")).unwrap_or(false)
            });
            
            if show_modal {
                // Create modal window
                let modal_id = egui::Id::new("logger_colors_modal");
                egui::Window::new("Logger Colors")
                    .id(modal_id)
                    .default_size(egui::Vec2::new(450.0, 650.0))  // Adjusted height for better initial view
                    .min_size(egui::Vec2::new(350.0, 550.0))      // Adjusted min height for better initial view
                    .collapsible(false)
                    .resizable(true)
                    .title_bar(true)
                    .show(ui.ctx(), |ui| {
                        // Get a copy of the colors first
                        let mut colors = colors_dynamic.get();
                        
                        let mut changed = false;
                        
                        // Store sync state in memory to persist between frames
                        let mut sync_colors = ui.ctx().memory_mut(|mem| {
                            mem.data.get_temp::<bool>(egui::Id::new("logger_colors_sync_mode"))
                                .unwrap_or(false)
                        });
                        
                        ui.horizontal(|ui| {
                            ui.heading("Log Colors");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.checkbox(&mut sync_colors, "Match Log & Messages").changed() {
                                    // Update sync state in memory
                                    ui.ctx().memory_mut(|mem| {
                                        mem.data.insert_temp(egui::Id::new("logger_colors_sync_mode"), sync_colors);
                                    });
                                }
                            });
                        });
                        ui.add_space(8.0);
                        
                        // Standard Log Types Section
                        egui::Frame::group(ui.style())
                            .fill(ui.style().visuals.window_fill)
                            .show(ui, |ui| {
                                ui.heading("Standard Log Types");
                                ui.add_space(8.0);
                                
                                // Two-column layout
                                ui.columns(2, |columns| {
                                    // Left column: Log Levels
                                    columns[0].group(|ui| {
                                        ui.heading("Log Levels");
                                        ui.add_space(4.0);
                                        
                                        let label_width = 60.0;  // Reduced from 70.0
                                        
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("INFO:"));
                                                if ui.color_edit_button_srgba(&mut colors.info_level).changed() {
                                                    changed = true;
                                                    // Also update the legacy field for backward compatibility
                                                    colors.info = colors.info_level;
                                                    
                                                    // If sync mode is on, also update the message color
                                                    if sync_colors {
                                                        colors.info_message = colors.info_level;
                                                    }
                                                }
                                            });
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("WARNING:"));
                                                if ui.color_edit_button_srgba(&mut colors.warning_level).changed() {
                                                    changed = true;
                                                    // Also update the legacy field for backward compatibility
                                                    colors.warning = colors.warning_level;
                                                    
                                                    // If sync mode is on, also update the message color
                                                    if sync_colors {
                                                        colors.warning_message = colors.warning_level;
                                                    }
                                                }
                                            });
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("ERROR:"));
                                                if ui.color_edit_button_srgba(&mut colors.error_level).changed() {
                                                    changed = true;
                                                    // Also update the legacy field for backward compatibility
                                                    colors.error = colors.error_level;
                                                    
                                                    // If sync mode is on, also update the message color
                                                    if sync_colors {
                                                        colors.error_message = colors.error_level;
                                                    }
                                                }
                                            });
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("DEBUG:"));
                                                if ui.color_edit_button_srgba(&mut colors.debug_level).changed() {
                                                    changed = true;
                                                    // Also update the legacy field for backward compatibility
                                                    colors.debug = colors.debug_level;
                                                    
                                                    // If sync mode is on, also update the message color
                                                    if sync_colors {
                                                        colors.debug_message = colors.debug_level;
                                                    }
                                                }
                                            });
                                        });
                                    });
                                    
                                    // Right column: Messages
                                    columns[1].group(|ui| {
                                        ui.heading("Messages");
                                        ui.add_space(4.0);
                                        
                                        let label_width = 60.0;  // Reduced from 70.0
                                        
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("INFO:"));
                                                if ui.color_edit_button_srgba(&mut colors.info_message).changed() {
                                                    changed = true;
                                                    
                                                    // If sync mode is on, also update the level color
                                                    if sync_colors {
                                                        colors.info_level = colors.info_message;
                                                        colors.info = colors.info_level; // Also update legacy field
                                                    }
                                                }
                                            });
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("WARNING:"));
                                                if ui.color_edit_button_srgba(&mut colors.warning_message).changed() {
                                                    changed = true;
                                                    
                                                    // If sync mode is on, also update the level color
                                                    if sync_colors {
                                                        colors.warning_level = colors.warning_message;
                                                        colors.warning = colors.warning_level; // Also update legacy field
                                                    }
                                                }
                                            });
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("ERROR:"));
                                                if ui.color_edit_button_srgba(&mut colors.error_message).changed() {
                                                    changed = true;
                                                    
                                                    // If sync mode is on, also update the level color
                                                    if sync_colors {
                                                        colors.error_level = colors.error_message;
                                                        colors.error = colors.error_level; // Also update legacy field
                                                    }
                                                }
                                            });
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("DEBUG:"));
                                                if ui.color_edit_button_srgba(&mut colors.debug_message).changed() {
                                                    changed = true;
                                                    
                                                    // If sync mode is on, also update the level color
                                                    if sync_colors {
                                                        colors.debug_level = colors.debug_message;
                                                        colors.debug = colors.debug_level; // Also update legacy field
                                                    }
                                                }
                                            });
                                        });
                                    });
                                });
                            });
                        
                        ui.add_space(8.0);
                        
                        // Custom Log Types Section
                        egui::Frame::group(ui.style())
                            .fill(ui.style().visuals.window_fill)
                            .show(ui, |ui| {
                                ui.heading("Custom Log Types");
                                ui.add_space(8.0);
                                
                                // Get all custom log types
                                let mut custom_identifiers: Vec<String> = colors.custom_colors.keys().cloned().collect();
                                custom_identifiers.sort(); // Sort them alphabetically
                                
                                // Store the total count for UI feedback
                                let total_count = custom_identifiers.len();
                                
                                if custom_identifiers.is_empty() {
                                    ui.label("No custom log types defined yet. Add one below.");
                                } else {
                                    ui.label(format!("Total custom types: {} (scroll to see all)", total_count));
                                    ui.add_space(4.0);
                                    
                                    // Calculate available height for the scrollable area
                                    let available_height = ui.available_height().min(200.0);
                                    
                                    // Two-column layout with scrollable area
                                    ui.columns(2, |columns| {
                                        // Left column: Log Levels
                                        columns[0].group(|ui| {
                                            ui.heading("Log Levels");
                                            ui.add_space(4.0);
                                            
                                            let label_width = 70.0;
                                            
                                            // Create a scrollable area for the level colors with visual indicators
                                            egui::ScrollArea::vertical()
                                                .max_height(available_height)
                                                .auto_shrink([false, false])
                                                .id_salt("custom_levels_scroll")
                                                .show(ui, |ui| {
                                                    // Add visual indicator for scrolling
                                                    ui.visuals_mut().widgets.noninteractive.bg_stroke.width = 1.0;
                                                    for identifier in &custom_identifiers {
                                                        if let Some(wrapper) = colors.custom_colors.get_mut(identifier) {
                                                            ui.horizontal(|ui| {
                                                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                                    ui.add_sized([label_width, 20.0], egui::Label::new(format!("{}:", identifier.to_uppercase())));
                                                                    if ui.color_edit_button_srgba(&mut wrapper.level_color).changed() {
                                                                        changed = true;
                                                                        
                                                                        // If sync mode is on, also update the message color
                                                                        if sync_colors {
                                                                            wrapper.message_color = wrapper.level_color;
                                                                        }
                                                                    }
                                                                });
                                                            });
                                                        }
                                                    }
                                                });
                                        });
                                        
                                        // Right column: Messages
                                        columns[1].group(|ui| {
                                            ui.heading("Messages");
                                            ui.add_space(4.0);
                                            
                                            let label_width = 70.0;
                                            
                                            // Create a scrollable area for the message colors with visual indicators
                                            egui::ScrollArea::vertical()
                                                .max_height(available_height)
                                                .auto_shrink([false, false])
                                                .id_salt("custom_messages_scroll")
                                                .show(ui, |ui| {
                                                    // Add visual indicator for scrolling
                                                    ui.visuals_mut().widgets.noninteractive.bg_stroke.width = 1.0;
                                                    for identifier in &custom_identifiers {
                                                        if let Some(wrapper) = colors.custom_colors.get_mut(identifier) {
                                                            ui.horizontal(|ui| {
                                                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                                    ui.add_sized([label_width, 20.0], egui::Label::new(format!("{}:", identifier.to_uppercase())));
                                                                    if ui.color_edit_button_srgba(&mut wrapper.message_color).changed() {
                                                                        changed = true;
                                                                        
                                                                        // If sync mode is on, also update the level color
                                                                        if sync_colors {
                                                                            wrapper.level_color = wrapper.message_color;
                                                                        }
                                                                    }
                                                                });
                                                            });
                                                        }
                                                    }
                                                });
                                        });
                                    });
                                }
                            });
                        
                        ui.add_space(8.0);
                        
                        // System Colors Section
                        egui::Frame::group(ui.style())
                            .fill(ui.style().visuals.window_fill)
                            .show(ui, |ui| {
                                ui.heading("System Colors");
                                ui.add_space(4.0);
                                
                                // Two-column layout for system colors
                                ui.columns(2, |columns| {
                                    let label_width = 70.0;  // Reduced from 90.0
                                    
                                    // Left column
                                    columns[0].group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("TIMESTAMP:"));
                                                changed |= ui.color_edit_button_srgba(&mut colors.timestamp).changed();
                                            });
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("DEFAULT:"));
                                                changed |= ui.color_edit_button_srgba(&mut colors.default).changed();
                                            });
                                        });
                                    });
                                    
                                    // Right column
                                    columns[1].group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("SYSTEM:"));
                                                changed |= ui.color_edit_button_srgba(&mut colors.system).changed();
                                            });
                                        });
                                        
                                        ui.horizontal(|ui| {
                                            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                                ui.add_sized([label_width, 20.0], egui::Label::new("SUCCESS:"));
                                                changed |= ui.color_edit_button_srgba(&mut colors.success).changed();
                                            });
                                        });
                                    });
                                });
                            });
                            
                        // Apply changes if needed
                        if changed {
                            // Update shared colors
                            colors_dynamic.set(colors.clone());
                            
                            // Save colors to file with correct path for gerber_viewer
                            Self::save_colors_for_gerber_viewer(&colors);
                        }
                        
                        ui.add_space(8.0);
                        
                        // Add New Custom Type Section
                        egui::Frame::group(ui.style())
                            .fill(ui.style().visuals.window_fill)
                            .show(ui, |ui| {
                                ui.heading("Add New Custom Log Type");
                                ui.add_space(4.0);
                                
                                // Static storage for the new custom type name
                                let mut new_custom_type = ui.ctx().memory_mut(|mem| {
                                    mem.data.get_temp::<String>(egui::Id::new("new_custom_log_type"))
                                        .unwrap_or_default()
                                });
                                
                                // Input for new custom type name
                                ui.horizontal(|ui| {
                                    ui.label("Type name:");
                                    let edit_response = ui.text_edit_singleline(&mut new_custom_type);
                                    
                                    if edit_response.changed() {
                                        ui.ctx().memory_mut(|mem| {
                                            mem.data.insert_temp(egui::Id::new("new_custom_log_type"), new_custom_type.clone());
                                        });
                                    }
                                    
                                    // Add by pressing Enter
                                    let add_type = edit_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) || 
                                                   ui.button("Add Type").clicked();
                                    
                                    if add_type && !new_custom_type.is_empty() {
                                        // Make sure the type name is cleaned
                                        let type_name = new_custom_type.to_lowercase().trim().to_string();
                                        
                                        if !type_name.is_empty() && !colors.custom_colors.contains_key(&type_name) {
                                            // Add the new custom type with default colors
                                            let level_color = egui::Color32::from_rgb(200, 200, 200);
                                            let message_color = egui::Color32::from_rgb(255, 255, 255);
                                            
                                            colors.custom_colors.insert(type_name.clone(), crate::logger_colors::Color32Wrapper {
                                                level_color,
                                                message_color,
                                            });
                                            
                                            // Update shared colors immediately
                                            colors_dynamic.set(colors.clone());
                                            
                                            // Clear the input
                                            new_custom_type.clear();
                                            ui.ctx().memory_mut(|mem| {
                                                mem.data.insert_temp(egui::Id::new("new_custom_log_type"), String::new());
                                            });
                                        }
                                    }
                                });
                                
                                ui.add_space(4.0);
                                
                                // Add some example usage instructions
                                ui.label("Example: Add 'network' to log network-related messages");
                                ui.label("Use: logger.log_custom(\"network\", \"Connected to server\")");
                            });
                            
                        ui.add_space(8.0);
                        
                        // Buttons section
                        egui::Frame::group(ui.style())
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    // Close button
                                    if ui.button("Close").clicked() {
                                        ui.ctx().memory_mut(|mem| {
                                            mem.data.remove::<bool>(egui::Id::new("show_logger_colors_modal"));
                                        });
                                    }
                                    
                                    // Use a spacer to push other buttons to the right
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        // Reset Defaults button
                                        if ui.button("Reset Defaults").clicked() {
                                            // Create default colors
                                            let default_colors = LogColors::default();
                                            
                                            // Update shared colors immediately
                                            colors_dynamic.set(default_colors.clone());
                                            colors = default_colors;
                                            changed = true; // Mark as changed to force refresh
                                        }
                                        
                                        // Add a small space between buttons
                                        ui.add_space(8.0);
                                        
                                        // Apply button
                                        if ui.button("Apply").clicked() {
                                            // Update shared colors immediately
                                            colors_dynamic.set(colors.clone());
                                            
                                            // Save colors to file with correct path for gerber_viewer
                                            Self::save_colors_for_gerber_viewer(&colors);
                                        }
                                    });
                                });
                            });
                    });
            }
        }
    }
    
    /// Displays the event log content with columns based on state
    fn show_event_log_content(&self, ui: &mut egui::Ui, state: &ReactiveEventLoggerState) {
        // Get column visibility settings
        let show_timestamps = state.show_timestamps;
        let show_log_level = state.show_log_level;
        let show_messages = state.show_messages;
        
        if !show_timestamps && !show_log_level && !show_messages {
            // Nothing to show
            ui.label("No columns selected");
            return;
        }
        
        // Fixed widths for timestamp and log level columns
        const TIMESTAMP_WIDTH: f32 = 190.0;
        const LEVEL_WIDTH: f32 = 100.0;
        
        // If we have custom colors, use rich text with the layout
        if let Some(colors_dynamic) = self.colors {
            // Get a copy of the colors from the Dynamic
            let colors = colors_dynamic.get();
            
            // Create a scrollable area for log content
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    // Create a table with headers
                    egui::Grid::new("logger_grid")
                        .num_columns(if show_timestamps { 1 } else { 0 } + 
                                     if show_log_level { 1 } else { 0 } + 
                                     if show_messages { 1 } else { 0 })
                        .striped(true)
                        .spacing([10.0, 4.0])
                        .min_col_width(0.0) // Allow us to fully control column widths
                        .show(ui, |ui| {
                            // Header row
                            if show_timestamps {
                                ui.add_sized([TIMESTAMP_WIDTH, 20.0], 
                                    egui::Label::new(egui::RichText::new("Timestamp").strong().size(14.0)));
                            }
                            
                            if show_log_level {
                                ui.add_sized([LEVEL_WIDTH, 20.0],
                                    egui::Label::new(egui::RichText::new("Level").strong().size(14.0)));
                            }
                            
                            if show_messages {
                                // Calculate available width for the message column header
                                let available_width = ui.available_width().max(300.0);
                                
                                // Make the header fill the available width
                                ui.scope(|ui| {
                                    ui.set_min_width(available_width);
                                    ui.label(egui::RichText::new("Message").strong().size(14.0));
                                });
                            }
                            
                            ui.end_row();
                            
                            // Create a separator spanning all columns
                            let col_count = (if show_timestamps { 1 } else { 0 }) + 
                                           (if show_log_level { 1 } else { 0 }) + 
                                           (if show_messages { 1 } else { 0 });
                            
                            for _ in 0..col_count {
                                ui.label("―――――――――――――");
                            }
                            ui.end_row();
                            
                            // Process logs in reverse order (newest first)
                            for log in state.logs.iter().rev() {
                                // Apply filter - skip logs that don't match the filter criteria
                                if !state.filter.should_display(log) {
                                    continue;
                                }
                                
                                if show_timestamps {
                                    let timestamp_text = egui::RichText::new(&log.timestamp.value.value)
                                        .color(colors.timestamp)
                                        .monospace();
                                    ui.add_sized([TIMESTAMP_WIDTH, 20.0], egui::Label::new(timestamp_text));
                                }
                                
                                if show_log_level {
                                    let (level_text, level_color) = get_log_level_text_and_color(log, &colors);
                                    ui.add_sized([LEVEL_WIDTH, 20.0], 
                                        egui::Label::new(
                                            egui::RichText::new(level_text)
                                            .color(level_color)
                                            .monospace()));
                                }
                                
                                if show_messages {
                                    // Format system info with consistent alignment
                                    let message_text = &log.log_message.content.value;
                                    let formatted_message = if message_text.contains("SYSTEM DETAILS") {
                                        format_system_info(message_text)
                                    } else {
                                        message_text.clone()
                                    };
                                    
                                    // Determine color based on log level first, then message content
                                    let message_color = if !log.log_level.info.value.is_empty() {
                                        // Check if it's a custom type
                                        if log.log_level.info.value.starts_with("CUSTOM:") {
                                            let identifier = log.log_level.info.value.strip_prefix("CUSTOM:").unwrap_or("");
                                            colors.get_custom_color_message(identifier)
                                        } else {
                                            colors.info_message
                                        }
                                    } else if !log.log_level.warning.value.is_empty() {
                                        colors.warning_message
                                    } else if !log.log_level.error.value.is_empty() {
                                        colors.error_message
                                    } else if !log.log_level.debug.value.is_empty() {
                                        colors.debug_message
                                    } else {
                                        // Fallback to content-based detection
                                        get_message_color(&formatted_message, &colors)
                                    };
                                    
                                    // Calculate available width to make the message column stretch
                                    let available_width = ui.available_width().max(300.0);
                                    
                                    // Create a label that fills the available width
                                    ui.scope(|ui| {
                                        ui.set_min_width(available_width);
                                        ui.add(egui::Label::new(
                                            egui::RichText::new(formatted_message)
                                                .color(message_color)
                                                .monospace()));
                                    });
                                }
                                
                                ui.end_row();
                            }
                        });
                });
            
            return;
        }
        
        // Fallback to plain text if colors are not available
        self.show_plain_text_logs(ui, state);
    }
    
    /// Fallback to plain text display when colors are not available
    fn show_plain_text_logs(&self, ui: &mut egui::Ui, state: &ReactiveEventLoggerState) {
        // Get column visibility settings
        let show_timestamps = state.show_timestamps;
        let show_log_level = state.show_log_level;
        let show_messages = state.show_messages;
        
        if !show_timestamps && !show_log_level && !show_messages {
            // Nothing to show
            ui.label("No columns selected");
            return;
        }
        
        // Calculate available height to fill the panel
        let available_height = ui.available_height();
        
        let mut log_text = String::new();
        
        // Process logs in reverse order (newest first)
        for log in state.logs.iter().rev() {
            // Apply filter - skip logs that don't match the filter criteria
            if !state.filter.should_display(log) {
                continue;
            }
            
            if show_timestamps {
                log_text.push_str(&format!("{} ", log.timestamp.value.value));
            }
            
            if show_log_level {
                // Find the non-empty log level
                if !log.log_level.info.value.is_empty() {
                    log_text.push_str(&format!("[{}] ", log.log_level.info.value));
                } else if !log.log_level.debug.value.is_empty() {
                    log_text.push_str(&format!("[{}] ", log.log_level.debug.value));
                } else if !log.log_level.warning.value.is_empty() {
                    log_text.push_str(&format!("[{}] ", log.log_level.warning.value));
                } else if !log.log_level.error.value.is_empty() {
                    log_text.push_str(&format!("[{}] ", log.log_level.error.value));
                }
            }
            
            if show_messages {
                log_text.push_str(&log.log_message.content.value);
            }
            
            log_text.push('\n');
        }
        
        // Create a scrollable area for the plain text content
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .stick_to_bottom(true)
            .show(ui, |ui| {
                // Show the logs in a monospace, non-interactive text editor that fills the space
                egui::TextEdit::multiline(&mut log_text)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .min_size(egui::vec2(ui.available_width(), available_height))
                    .interactive(false)
                    .show(ui);
            });
    }
}

// Helper function to get log level text and color
pub fn get_log_level_text_and_color(log: &LoggerPayload, colors: &LogColors) -> (String, egui::Color32) {
    if !log.log_level.info.value.is_empty() {
        // Check if it's a custom type (starts with "CUSTOM:")
        if log.log_level.info.value.starts_with("CUSTOM:") {
            let identifier = log.log_level.info.value.strip_prefix("CUSTOM:").unwrap_or("");
            (format!("[CUSTOM:{}]", identifier), colors.get_custom_color_level(identifier))
        } else {
            (format!("[{}]", log.log_level.info.value), colors.info_level)
        }
    } else if !log.log_level.debug.value.is_empty() {
        (format!("[{}]", log.log_level.debug.value), colors.debug_level)
    } else if !log.log_level.warning.value.is_empty() {
        (format!("[{}]", log.log_level.warning.value), colors.warning_level)
    } else if !log.log_level.error.value.is_empty() {
        (format!("[{}]", log.log_level.error.value), colors.error_level)
    } else {
        (String::new(), colors.default)
    }
}

// Helper function to get message color
pub fn get_message_color(message_text: &str, colors: &LogColors) -> egui::Color32 {
    // Determine message type based on content
    if message_text.contains("[INFO]") || message_text.contains("[SUCCESS]") {
        colors.info_message
    } else if message_text.contains("[WARNING]") {
        colors.warning_message
    } else if message_text.contains("[ERROR]") {
        colors.error_message
    } else if message_text.contains("[DEBUG]") {
        colors.debug_message
    } else if message_text.contains("[CUSTOM:") {
        // Extract the custom identifier from format like "[CUSTOM:mytype]"
        if let Some(start) = message_text.find("[CUSTOM:") {
            if let Some(end) = message_text[start..].find("]") {
                let custom_type = &message_text[start + 8..start + end];
                return colors.get_custom_color_message(custom_type);
            }
        }
        colors.default
    } else {
        // Default for other message types
        colors.default
    }
}

// Helper function to format system info
pub fn format_system_info(message: &str) -> String {
    // Split the message into lines and align key-value pairs
    message
        .lines()
        .map(|line| {
            if let Some((key, value)) = line.split_once(':') {
                format!("{:<20}: {}", key.trim(), value.trim()) // Align keys to 20 characters
            } else {
                line.to_string() // Return the line as-is if no colon is found
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// Helper function to check if any filters are active
pub fn is_any_filter_active(filter: &LogFilter) -> bool {
    // Check if any log type filter is turned off
    !filter.show_info || 
    !filter.show_warning || 
    !filter.show_error || 
    !filter.show_debug || 
    !filter.show_custom || 
    !filter.show_system ||
    // Check if text filter is active
    !filter.text_filter.is_empty()
}