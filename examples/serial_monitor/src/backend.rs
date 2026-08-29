//! Serial worker thread — the backend of this app.
//!
//! Adapted from SaturnGridSim's serial worker. Commands travel UI →
//! worker over an `mpsc` channel (the dispatch path). Results travel
//! worker → UI by writing shared `Dynamic` cells directly — `Dynamic<T>`
//! is `Arc<Mutex<T>>` underneath, so clones handed to the thread at
//! spawn are all it needs; Path A carries the values to the citizens on
//! the next frame. No egui state is touched off-thread; the worker only
//! calls `ctx.request_repaint()` to wake the UI.

use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc;
use std::time::Duration;

use eframe::egui;
use egui_mobius_reactive::Dynamic;

/// Cap on the console buffer so long sessions stay bounded.
const MAX_RX_LINES: usize = 2000;

/// Commands from the UI to the worker.
pub enum SerialCommand {
    /// Write the line plus a trailing newline to the port.
    Line(String),
}

/// UI-side handle to the worker. Owns the command sender; dropping it
/// (or calling [`disconnect`](Self::disconnect)) ends the worker loop.
pub struct SerialBackend {
    ctx: egui::Context,
    cmd_tx: Option<mpsc::Sender<SerialCommand>>,
}

impl SerialBackend {
    pub fn new(ctx: egui::Context) -> Self {
        Self { ctx, cmd_tx: None }
    }

    /// Spawn the worker for `port` at `baud`. The worker reports through
    /// the `connected` / `status` / `rx_lines` cells.
    pub fn connect(
        &mut self,
        port: &str,
        baud: u32,
        connected: Dynamic<bool>,
        status: Dynamic<String>,
        rx_lines: Dynamic<Vec<String>>,
    ) {
        // A previous worker (if any) shuts down when its sender drops.
        self.cmd_tx = None;

        let (cmd_tx, cmd_rx) = mpsc::channel::<SerialCommand>();
        let port = port.to_string();
        let ctx = self.ctx.clone();

        std::thread::Builder::new()
            .name("serial-worker".into())
            .spawn(move || {
                worker_loop(&port, baud, cmd_rx, connected, status, rx_lines, ctx);
            })
            .expect("failed to spawn serial worker thread");

        self.cmd_tx = Some(cmd_tx);
    }

    /// Drop the command sender; the worker notices on its next loop
    /// iteration, closes the port, and exits.
    pub fn disconnect(&mut self) {
        self.cmd_tx = None;
    }

    /// Queue one line for transmission. Returns false if not connected.
    pub fn send(&self, line: String) -> bool {
        match &self.cmd_tx {
            Some(tx) => tx.send(SerialCommand::Line(line)).is_ok(),
            None => false,
        }
    }
}

fn push_capped(cell: &Dynamic<Vec<String>>, line: String) {
    let mut buf = cell.get();
    buf.push(line);
    if buf.len() > MAX_RX_LINES {
        let drop = buf.len() - MAX_RX_LINES;
        buf.drain(0..drop);
    }
    cell.set(buf);
}

fn worker_loop(
    port_name: &str,
    baud: u32,
    cmd_rx: mpsc::Receiver<SerialCommand>,
    connected: Dynamic<bool>,
    status: Dynamic<String>,
    rx_lines: Dynamic<Vec<String>>,
    ctx: egui::Context,
) {
    let port = serialport::new(port_name, baud)
        .timeout(Duration::from_millis(90))
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::None)
        .stop_bits(serialport::StopBits::One)
        .open();

    let port = match port {
        Ok(p) => {
            connected.set(true);
            status.set(format!("connected: {port_name} @ {baud}"));
            ctx.request_repaint();
            p
        }
        Err(e) => {
            status.set(format!("failed to open {port_name}: {e}"));
            ctx.request_repaint();
            return;
        }
    };

    // Read and write independently: clone the port handle for the
    // writer, wrap the original in a BufReader for line reads.
    let mut writer = match port.try_clone() {
        Ok(w) => w,
        Err(e) => {
            connected.set(false);
            status.set(format!("failed to clone port handle: {e}"));
            ctx.request_repaint();
            return;
        }
    };
    let mut reader = BufReader::new(port);
    let mut line_buf = String::new();

    let finish = |why: String| {
        connected.set(false);
        status.set(why);
        ctx.request_repaint();
    };

    loop {
        // Transmit anything the UI queued.
        match cmd_rx.try_recv() {
            Ok(SerialCommand::Line(line)) => {
                if let Err(e) = writer
                    .write_all(line.as_bytes())
                    .and_then(|_| writer.write_all(b"\n"))
                {
                    push_capped(&rx_lines, format!("[tx error] {e}"));
                    ctx.request_repaint();
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                // UI dropped the sender — orderly shutdown.
                finish("disconnected".into());
                return;
            }
            Err(mpsc::TryRecvError::Empty) => {}
        }

        // Read one line; the 90 ms timeout keeps the loop responsive to
        // commands and shutdown even on a silent port.
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(0) => {
                finish("port closed (EOF)".into());
                return;
            }
            Ok(_) => {
                let trimmed = line_buf.trim_end();
                if trimmed.is_empty() {
                    continue;
                }
                push_capped(&rx_lines, trimmed.to_string());
                ctx.request_repaint();
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
            // Some platforms surface the timeout as WouldBlock instead.
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => continue,
            Err(e) => {
                finish(format!("read error: {e}"));
                return;
            }
        }
    }
}
