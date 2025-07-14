//! EguiMobiusEventLogger
//! 
//! **Description**
//! 
//! This file contains the EguiMobiusEventLogger struct and its implementation.
//! A specialized component for logging and displaying UI events in Egui applications.
//! Uses the egui_mobius signal/slot mechanism for communication.
//! 
//! **Contents**
//! 
//! - EguiMobiusEventLogger struct
//! - Logging methods
//! - UI rendering
//!  
use eframe::egui;
use egui_mobius::{Dispatcher, SignalDispatcher, Signal, Slot};

use crate::components::event_logger::log_colors::LogColors;
use crate::components::event_logger::log_type::LogType;
use crate::components::event_logger::messages::{LogEntry, LoggerEvent, LoggerResponse, Message, LogSender};
use crate::components::event_logger::logger_state::LoggerState;

/// The main event logger component
#[allow(dead_code)]
#[derive(Clone)]
pub struct EguiMobiusEventLogger {
    /// The dispatcher for sending events
    dispatcher: Dispatcher<LoggerEvent>,
    /// The slot for receiving responses
    response_slot: Option<Slot<LoggerResponse>>,
    /// The UI context
    ctx: egui::Context,
}

impl Default for EguiMobiusEventLogger {
    fn default() -> Self {
        // Create a new dispatcher and context
        let dispatcher = Dispatcher::<LoggerEvent>::new();
        let ctx = egui::Context::default();
        
        // Initialize the shared state with default colors
        crate::components::event_logger::processor::init_logger_backend(LogColors::default());
        
        // Create a new logger with default settings
        Self::new(ctx, LogColors::default(), dispatcher, None)
    }
}

impl EguiMobiusEventLogger {
    /// Create a new event logger with the given context, colors, and dispatcher
    pub fn new(
        ctx: egui::Context, 
        colors: LogColors, 
        dispatcher: Dispatcher<LoggerEvent>,
        response_slot: Option<Slot<LoggerResponse>>
    ) -> Self {
        // Initialize the shared state
        crate::components::event_logger::processor::init_logger_backend(colors.clone());
        
        // Register the response handler if provided
        if let Some(mut slot) = response_slot.clone() {
            let ctx_clone = ctx.clone();
            
            slot.start(move |_response| {
                // The state is already updated in the processor
                // We just need to request a repaint
                ctx_clone.request_repaint();
            });
        }
        
        // Register slot for logger events
        dispatcher.register_slot("logger_events", move |_event| {
            // Events are handled by the processor
        });
        
        Self {
            dispatcher,
            response_slot,
            ctx,
        }
    }
    
    /// Add a new log entry
    pub fn add_log(&self, msg: Message, sender: LogSender, style_type: LogType) {
        self.dispatcher.send("logger_events", LoggerEvent::AddEntry(msg, sender, style_type));
    }
    
    /// Convenience methods for different log levels
    pub fn info(&self, msg: String, sender: LogSender, style_type: LogType) {
        self.add_log(Message::Info(msg), sender, style_type);
    }
    
    pub fn warn(&self, msg: String, sender: LogSender, style_type: LogType) {
        self.add_log(Message::Warn(msg), sender, style_type);
    }
    
    pub fn debug(&self, msg: String, sender: LogSender, style_type: LogType) {
        self.add_log(Message::Debug(msg), sender, style_type);
    }
    
    pub fn error(&self, msg: String, sender: LogSender, style_type: LogType) {
        self.add_log(Message::Error(msg), sender, style_type);
    }
    
    /// Clears all log entries from the logger
    pub fn clear(&self) {
        self.dispatcher.send("logger_events", LoggerEvent::ClearLog);
    }
    
    /// Updates the color scheme for the terminal
    pub fn update_colors(&self, new_colors: LogColors) {
        self.dispatcher.send("logger_events", LoggerEvent::UpdateColors(new_colors));
    }
    
    /// Toggle visibility of timestamps column
    pub fn toggle_timestamps(&self, show: bool) {
        self.dispatcher.send("logger_events", LoggerEvent::ToggleTimestamps(show));
    }
    
    /// Toggle visibility of messages column
    pub fn toggle_messages(&self, show: bool) {
        self.dispatcher.send("logger_events", LoggerEvent::ToggleMessages(show));
    }
    
    /// Export recent log entries for backup/restoration
    pub fn export_recent(&self, count: usize) -> Vec<LogEntry> {
        // Send the event
        self.dispatcher.send("logger_events", LoggerEvent::ExportRecent(count));
        
        // Return the current entries from the shared state
        let state = crate::components::event_logger::processor::LOGGER_STATE.lock().unwrap();
        state.export_recent(count)
    }
    
    /// Show the event logger UI
    pub fn show(&self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Display column visibility controls
            ui.horizontal(|ui| {
                ui.label("Display columns:");
                
                // Get current values from shared state
                let (show_timestamps, show_messages) = {
                    let state = crate::components::event_logger::processor::LOGGER_STATE.lock().unwrap();
                    (state.show_timestamps, state.show_messages)
                };
                
                // Timestamps checkbox
                let mut ts_value = show_timestamps;
                if ui.checkbox(&mut ts_value, "Timestamps").changed() {
                    // Send the event if changed
                    self.toggle_timestamps(ts_value);
                }
                
                // Messages checkbox
                let mut msg_value = show_messages;
                if ui.checkbox(&mut msg_value, "Messages").changed() {
                    // Send the event if changed
                    self.toggle_messages(msg_value);
                }
            });
            
            // Display terminal content - get a fresh lock on shared state
            let state = crate::components::event_logger::processor::LOGGER_STATE.lock().unwrap();
            self.show_event_log_content(ui, &state);
        });
    }
    
    /// Displays the event log content with two columns
    fn show_event_log_content(&self, ui: &mut egui::Ui, state: &LoggerState) {
        // Get column visibility settings
        let show_timestamps = state.show_timestamps;
        let show_messages = state.show_messages;
        
        // Choose layout based on visible columns
        if show_timestamps && show_messages {
            // Show both columns in a table
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    // Two-column layout
                    egui::Grid::new("logger_grid")
                        .num_columns(2)
                        .spacing([10.0, 6.0])
                        .striped(true)
                        .show(ui, |ui| {
                            // Add headers
                            ui.strong("Time");
                            ui.strong("Message");
                            ui.end_row();
                            
                            // Add entries
                            for entry in state.logs.iter().rev() {
                                let (timestamp, message) = state.format_log_entry(entry);
                                
                                ui.label(timestamp);
                                ui.label(message);
                                ui.end_row();
                            }
                        });
                });
        } else if show_timestamps {
            // Show only timestamps
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in state.logs.iter().rev() {
                        let (timestamp, _) = state.format_log_entry(entry);
                        ui.label(timestamp);
                    }
                });
        } else if show_messages {
            // Show only messages
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for entry in state.logs.iter().rev() {
                        let (_, message) = state.format_log_entry(entry);
                        ui.label(message);
                    }
                });
        } else {
            // Nothing to show
            ui.label("No columns selected");
        }
    }
}

/// Factory function to create a logger with signal/slot
pub fn create_event_logger(ctx: egui::Context, colors: LogColors) -> (EguiMobiusEventLogger, Slot<LoggerEvent>, Signal<LoggerResponse>) {
    // Create signal and slot
    let (_event_signal, event_slot) = egui_mobius::factory::create_signal_slot::<LoggerEvent>();
    let (response_signal, response_slot) = egui_mobius::factory::create_signal_slot::<LoggerResponse>();
    
    // Initialize the shared state
    crate::components::event_logger::processor::init_logger_backend(colors.clone());
    
    // Create dispatcher
    let dispatcher = Dispatcher::<LoggerEvent>::new();
    
    // Register the event handler
    let response_signal_clone = response_signal.clone();
    dispatcher.register_slot("logger_events", move |event| {
        let response = crate::components::event_logger::processor::process_event(event);
        if let Err(e) = response_signal_clone.send(response) {
            eprintln!("Failed to send logger response: {e:?}");
        }
    });
    
    // Create the logger
    let logger = EguiMobiusEventLogger::new(ctx, colors, dispatcher, Some(response_slot));
    
    (logger, event_slot, response_signal)
}