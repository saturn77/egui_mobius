// Re-export the main components from the event logger

// Import and re-export from log_colors
pub use super::log_colors::LogColors;

// Import and re-export from log_type
pub use super::log_type::LogType;

// Import and re-export from logger
pub use super::logger::{EguiMobiusEventLogger, create_event_logger};

// Import and re-export from logger_state
pub use super::logger_state::LoggerState;

// Import and re-export from messages
pub use super::messages::{
    Message, 
    LogEntry,
    LogSender,
};

// Import and re-export from serialization
pub use super::serialization::color32_serde::{serialize, deserialize};

// Import and re-export from platform
pub use super::platform::{
    banner::Banner, 
    details::Details,
};