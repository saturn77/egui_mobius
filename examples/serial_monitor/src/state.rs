//! Shared application state — the reactive cells the citizens and the
//! serial worker communicate through.
//!
//! Ownership discipline (one writer per cell):
//! - `ports`     — written by `monitor_actions` (RefreshPorts), read by Monitor.
//! - `connected` — written by the serial worker, read by Monitor.
//! - `status`    — written by the serial worker, read by Monitor.
//! - `rx_lines`  — written by the serial worker, read by Console.
//! - `log`       — written by the drain loop and actions, read by Logger.
//! - `line_ending` — written by Monitor, read by `monitor_actions` at send.

use egui_mobius_reactive::Dynamic;

use crate::backend::LineEnding;

pub struct SharedState {
    /// Serial ports discovered by the last RefreshPorts.
    pub ports: Dynamic<Vec<String>>,
    /// Whether the worker currently holds an open port.
    pub connected: Dynamic<bool>,
    /// One-line connection status for the Monitor panel.
    pub status: Dynamic<String>,
    /// Received lines, capped — the Console renders these.
    pub rx_lines: Dynamic<Vec<String>>,
    /// App log — citizen lifecycle + TX/RX events, capped.
    pub log: Dynamic<Vec<String>>,
    /// TX line terminator — written by the Monitor panel, read at send.
    pub line_ending: Dynamic<LineEnding>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            ports: Dynamic::new(Vec::new()),
            connected: Dynamic::new(false),
            status: Dynamic::new("disconnected".into()),
            rx_lines: Dynamic::new(Vec::new()),
            log: Dynamic::new(Vec::new()),
            line_ending: Dynamic::new(LineEnding::default()),
        }
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}
