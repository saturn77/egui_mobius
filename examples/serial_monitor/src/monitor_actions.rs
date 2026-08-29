//! Actions for the Monitor citizen: its outbox messages, routed to the
//! serial backend. This is the dispatch path — the only place the word
//! applies. Results never come back through here: the worker writes the
//! shared `Dynamic` cells directly and Path A carries them to the
//! Console and Monitor citizens.

use crate::backend::SerialBackend;
use crate::citizens::append_log;
use crate::messages::AppMessage;
use crate::state::SharedState;

pub fn handle(msg: AppMessage, state: &SharedState, backend: &mut SerialBackend) {
    match msg {
        AppMessage::RefreshPorts => match serialport::available_ports() {
            Ok(ports) => {
                let names: Vec<String> =
                    ports.into_iter().map(|p| p.port_name).collect();
                append_log(&state.log, format!("[ports] {} found", names.len()));
                state.ports.set(names);
            }
            Err(e) => {
                append_log(&state.log, format!("[ports] enumeration failed: {e}"));
                state.ports.set(Vec::new());
            }
        },
        AppMessage::Connect { port, baud } => {
            append_log(&state.log, format!("[serial] connecting {port} @ {baud}"));
            state.status.set(format!("connecting {port}…"));
            backend.connect(
                &port,
                baud,
                state.connected.clone(),
                state.status.clone(),
                state.rx_lines.clone(),
            );
        }
        AppMessage::Disconnect => {
            append_log(&state.log, "[serial] disconnect requested".into());
            backend.disconnect();
        }
        AppMessage::Send(line) => {
            if backend.send(line.clone()) {
                append_log(&state.log, format!("[tx] {line}"));
            } else {
                append_log(&state.log, "[tx] dropped — not connected".into());
            }
        }
        AppMessage::ClearRx => {
            state.rx_lines.set(Vec::new());
            append_log(&state.log, "[console] cleared".into());
        }
    }
}
