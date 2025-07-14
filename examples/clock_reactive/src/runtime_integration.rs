//! Runtime integration module
//!
//! This module contains the `RuntimeManager` struct which is responsible 
//! for managing the Mobius runtime and the background clock task. Basically
//! the runtime is started and a background task is started to update the
//! clock time every second. The runtime is used to handle messages from the
//! background task and update the UI accordingly.
use crate::{types::ClockMessage, state::AppState};
use egui_mobius::{EventRoute, MobiusRuntime, MobiusHandle};
use eframe::egui;
use std::sync::{Arc, mpsc};
use chrono::Local;
use tokio::sync::Notify;
use crate::types::LogEntry;

impl EventRoute for ClockMessage {
    fn route(&self) -> &str {
        match self {
            ClockMessage::TimeUpdated(_) => "time_updated",
            ClockMessage::Start => "start",
            ClockMessage::Stop  => "stop",
            ClockMessage::Clear => "clear",
        }
    }
}

pub struct RuntimeManager {
    runtime   : Option<tokio::task::JoinHandle<()>>,
    handle    : Option<Arc<MobiusHandle<ClockMessage>>>,
    shutdown  : Arc<Notify>,
    state     : Arc<AppState>,
}

impl RuntimeManager {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            runtime  : None,
            handle   : None,
            shutdown : Arc::new(Notify::new()),
            state,
        }
    }

    pub fn start(&mut self, _ctx: egui::Context) {
        if self.runtime.is_some() {
            return;
        }

        let (runtime, handle, _processed_rx) = MobiusRuntime::new();
        let handle = Arc::new(handle);
        let _handle_clone = handle.clone();
        let shutdown = self.shutdown.clone();



        // Start clock updates in a separate tokio task
        // Since the clock is updated here, format the time for the UI
        // and then the reactive state management will take care of the
        // rest.
        let current_time = self.state.current_time.clone().to_owned();  // Create owned Dynamic
        let use_24h = self.state.use_24h.clone().to_owned();  // Create owned Dynamic
        let logs = self.state.logs.clone().to_owned(); // Create owned Dynamic
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let now = chrono::Local::now();
                let time_str = if use_24h.get() {
                    now.format("%H:%M:%S").to_string()
                } else {
                    now.format("%I:%M:%S %p").to_string().trim_start_matches('0').to_string()
                };
                current_time.set(time_str.clone());
                log::debug!("Time updated: {}", time_str);
                let mut current_logs = logs.get();
                if current_logs.len() >= 1000 {
                    current_logs.pop_front();
                }
                current_logs.push_back(LogEntry {
                    timestamp: Local::now(),
                    source: "clock".to_string(),
                    message: format!("Time updated: {time_str}"),
                    color: Some(egui::Color32::from_rgb(100, 200, 255)), // Light Blue
                });
                logs.set(current_logs);
            }
        });


        // The code below registers the message handlers for the runtime, 
        // and there is a placeholder for this messages in state.rs
        // Since the design is reactive, the state will be updated based on
        // the messages are not utilized at this moment, but are shown here 
        // for reference as aslo as a template for future use.

        // let state1 = self.state.clone();
        // runtime.register_handler("time_updated", move |msg| {
        //     let state = state1.clone();
        //     async move {
        //         log::debug!("Received time update message");
        //         state.handle_message(msg);
        //     }
        // });

        // let state2 = self.state.clone();
        // runtime.register_handler("start", move |msg| {
        //     let state = state2.clone();
        //     async move {
        //         state.handle_message(msg);
        //     }
        // });

        // let state3 = self.state.clone();
        // runtime.register_handler("stop", move |msg| {
        //     let state = state3.clone();
        //     async move {
        //         state.handle_message(msg);
        //     }
        // });

        // let state4 = self.state.clone();
        // runtime.register_handler("clear", move |msg| {
        //     let state = state4.clone();
        //     async move {
        //         state.handle_message(msg);
        //     }
        // });

        // Start runtime
        let rt = tokio::spawn(async move {
            runtime.run().await;
        });

        // Optional Control - Start background clock (presently not used)
        let (tx, _rx) = mpsc::channel();
        let shutdown_clone = shutdown.clone();
        tokio::spawn(async move {
            shutdown_clone.notified().await;
            let _ = tx.send(());
        });


        self.runtime = Some(rt);
        self.handle = Some(handle);
        
        // Optional Control - Start the clock in running state (presently not used)
        // if let Some(handle) = &self.handle {
        //     let _ = handle.send(ClockMessage::Start);
        // }
    }

    pub fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.shutdown();
        }
        self.shutdown.notify_one();
        if let Some(rt) = self.runtime.take() {
            rt.abort();
        }
    }
}

impl Drop for RuntimeManager {
    fn drop(&mut self) {
        self.stop();
    }
}
