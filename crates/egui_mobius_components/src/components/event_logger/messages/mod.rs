//! Signal/Slot messaging types for the event logger component
//!
//! This module defines the message types used for communication between the
//! UI and the logger backend using the egui_mobius signal/slot mechanism.

use crate::components::event_logger::log_colors::LogColors;
use crate::components::event_logger::log_type::LogType;
use chrono::{DateTime, Local};
use std::fmt::Debug;

/// Message types with different severity levels
#[derive(Clone, PartialEq, Debug)]
pub enum Message {
    Info(String),
    Warn(String),
    Debug(String),
    Error(String),
}

impl Message {
    /// Get the string content of the message
    pub fn content(&self) -> &str {
        match self {
            Message::Info(s) => s,
            Message::Warn(s) => s,
            Message::Debug(s) => s,
            Message::Error(s) => s,
        }
    }
    
    /// Get the message type as a string
    pub fn type_name(&self) -> &str {
        match self {
            Message::Info(_) => "INFO",
            Message::Warn(_) => "WARN",
            Message::Debug(_) => "DEBUG",
            Message::Error(_) => "ERROR",
        }
    }
}

/// Types of UI widgets that can generate messages
#[derive(Clone, Debug, PartialEq)]
pub enum UiWidgetType {
    Slider,
    Checkbox,
    ComboBox,
    RadioButton,
    Button,
    TextField,
    RichText,
    Panel,
    Tab,
    Canvas,
    System,
    Custom(String), // For extensibility
}

/// A sender represents the source of a log message
#[derive(Clone, Debug, PartialEq)]
pub struct LogSender {
    widget_type: UiWidgetType,
    id: Option<String>,    // Optional widget ID/name
}

impl LogSender {
    /// Create a new sender with the given widget type and ID
    pub fn new(widget_type: UiWidgetType, id: Option<String>) -> Self {
        Self { widget_type, id }
    }
    
    /// Create common sender types with convenience methods
    pub fn slider(id: impl Into<String>) -> Self {
        Self::new(UiWidgetType::Slider, Some(id.into()))
    }
    
    pub fn checkbox(id: impl Into<String>) -> Self {
        Self::new(UiWidgetType::Checkbox, Some(id.into()))
    }
    
    pub fn combo_box(id: impl Into<String>) -> Self {
        Self::new(UiWidgetType::ComboBox, Some(id.into()))
    }
    
    pub fn radio_button(id: impl Into<String>) -> Self {
        Self::new(UiWidgetType::RadioButton, Some(id.into()))
    }
    
    pub fn button(id: impl Into<String>) -> Self {
        Self::new(UiWidgetType::Button, Some(id.into()))
    }
    
    pub fn text_field(id: impl Into<String>) -> Self {
        Self::new(UiWidgetType::TextField, Some(id.into()))
    }
    
    pub fn rich_text(id: impl Into<String>) -> Self {
        Self::new(UiWidgetType::RichText, Some(id.into()))
    }
    
    pub fn panel(id: impl Into<String>) -> Self {
        Self::new(UiWidgetType::Panel, Some(id.into()))
    }
    
    pub fn tab(id: impl Into<String>) -> Self {
        Self::new(UiWidgetType::Tab, Some(id.into()))
    }
    
    pub fn canvas(id: impl Into<String>) -> Self {
        Self::new(UiWidgetType::Canvas, Some(id.into()))
    }
    
    pub fn system() -> Self {
        Self::new(UiWidgetType::System, None)
    }
    
    pub fn custom(name: impl Into<String>) -> Self {
        Self::new(UiWidgetType::Custom(name.into()), None)
    }
    
    /// Get the widget type name as a string
    pub fn type_name(&self) -> String {
        match &self.widget_type {
            UiWidgetType::Slider => "Slider".to_string(),
            UiWidgetType::Checkbox => "Checkbox".to_string(),
            UiWidgetType::ComboBox => "ComboBox".to_string(),
            UiWidgetType::RadioButton => "RadioButton".to_string(),
            UiWidgetType::Button => "Button".to_string(),
            UiWidgetType::TextField => "TextField".to_string(),
            UiWidgetType::RichText => "RichText".to_string(),
            UiWidgetType::Panel => "Panel".to_string(),
            UiWidgetType::Tab => "Tab".to_string(),
            UiWidgetType::Canvas => "Canvas".to_string(),
            UiWidgetType::System => "System".to_string(),
            UiWidgetType::Custom(name) => format!("Custom({})", name),
        }
    }
    
    /// Get a display name for the sender
    pub fn display_name(&self) -> String {
        match (&self.widget_type, &self.id) {
            (UiWidgetType::System, _) => "System".to_string(),
            (_, Some(id)) if !id.is_empty() => 
                format!("{}({})", self.type_name(), id),
            _ => self.type_name(),
        }
    }
}

/// A log entry contains timestamp, message, sender and visual styling type
#[derive(Clone, PartialEq, Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub message: Message,
    pub sender: LogSender,
    pub style_type: LogType,
}

/// Event types sent from UI to logger backend
#[derive(Clone, Debug)]
pub enum LoggerEvent {
    /// Add a new log entry
    AddEntry(Message, LogSender, LogType),
    /// Clear all log entries
    ClearLog,
    /// Update the color scheme
    UpdateColors(LogColors),
    /// Toggle timestamp display
    ToggleTimestamps(bool),
    /// Toggle message display
    ToggleMessages(bool),
    /// Export recent log entries
    ExportRecent(usize),
}

/// Response types sent from logger backend to UI
#[derive(Clone, Debug)]
pub enum LoggerResponse {
    /// A new entry was added
    EntryAdded(LogEntry),
    /// The log was cleared
    LogCleared,
    /// Colors were updated
    ColorsUpdated(LogColors),
    /// Timestamp display was toggled
    TimestampsToggled(bool),
    /// Message display was toggled
    MessagesToggled(bool),
    /// Recent entries were exported
    RecentExported(Vec<LogEntry>),
}