//! App-level messages — the intents the Monitor citizen pushes onto its
//! outbox, routed to the serial backend by `monitor_actions.rs`.

#[derive(Debug, Clone)]
pub enum AppMessage {
    /// Re-enumerate the system's serial ports.
    RefreshPorts,
    /// Open the named port at the given baud rate on the worker thread.
    Connect { port: String, baud: u32 },
    /// Close the port by shutting the worker down.
    Disconnect,
    /// Transmit one line (a `\n` is appended on the wire).
    Send(String),
    /// Clear the received-lines console.
    ClearRx,
}
