//! Logger Processor
//!
//! This module contains the backend processor for the event logger.
//! It receives events from the UI and sends responses back.

use chrono::Local;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;

use crate::components::event_logger::log_colors::LogColors;
use crate::components::event_logger::messages::{LogEntry, LoggerEvent, LoggerResponse};
use crate::components::event_logger::logger_state::LoggerState;

// Shared state for the logger backend
lazy_static! {
    pub static ref LOGGER_STATE: Arc<Mutex<LoggerState>> = Arc::new(Mutex::new(LoggerState::default()));
}

/// Initialize the logger backend with a given color scheme
pub fn init_logger_backend(colors: LogColors) {
    let mut state = LOGGER_STATE.lock().unwrap();
    *state = LoggerState::new(colors);
}

/// Process a logger event and return a response
pub fn process_event(event: LoggerEvent) -> LoggerResponse {
    match event {
        LoggerEvent::AddEntry(message, sender, style_type) => {
            let mut state = LOGGER_STATE.lock().unwrap();
            let entry = LogEntry {
                timestamp: Local::now(),
                message,
                sender,
                style_type,
            };
            
            // Add to state
            state.add_log(entry.clone());
            
            // Return response
            LoggerResponse::EntryAdded(entry)
        },
        LoggerEvent::ClearLog => {
            let mut state = LOGGER_STATE.lock().unwrap();
            state.clear();
            LoggerResponse::LogCleared
        },
        LoggerEvent::UpdateColors(colors) => {
            let mut state = LOGGER_STATE.lock().unwrap();
            state.update_colors(colors.clone());
            LoggerResponse::ColorsUpdated(colors)
        },
        LoggerEvent::ToggleTimestamps(show) => {
            let mut state = LOGGER_STATE.lock().unwrap();
            state.toggle_timestamps(show);
            LoggerResponse::TimestampsToggled(show)
        },
        LoggerEvent::ToggleMessages(show) => {
            let mut state = LOGGER_STATE.lock().unwrap();
            state.toggle_messages(show);
            LoggerResponse::MessagesToggled(show)
        },
        LoggerEvent::ExportRecent(count) => {
            let state = LOGGER_STATE.lock().unwrap();
            let entries = state.export_recent(count);
            LoggerResponse::RecentExported(entries)
        },
    }
}

/// Run the logger backend
/// 
/// This function starts a slot to handle logger events and send responses.
pub fn run_logger_backend(mut event_slot: Slot<LoggerEvent>, response_signal: Signal<LoggerResponse>) {
    event_slot.start(move |event| {
        let response = process_event(event);
        if let Err(e) = response_signal.send(response) {
            eprintln!("Failed to send logger response: {:?}", e);
        }
    });
}