use eframe::egui;
use egui_mobius::dispatching::{Dispatcher, SignalDispatcher};
use egui_mobius::factory;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;
use egui_mobius::types::Value;

use chrono::{DateTime, Local};
use lazy_static::lazy_static;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub enum Event {
    IncrementCounter,
    ResetCounter,
    Custom(String), // For logging/testing custom events
}

// src/response.rs
#[derive(Debug, Clone)]
pub enum Response {
    CounterUpdated(usize),
    Message(String), // For general-purpose backend messages
}

impl Default for Response {
    fn default() -> Self {
        Response::CounterUpdated(0)
    }
}

/// background_consumer_thread
///
/// This function runs on a dedicated thread and is responsible for:
/// - Receiving `Event` messages from the UI
/// - Processing them into `Response` values
/// - Sending `Response` values back to the UI thread
pub fn run_backend(mut event_slot: Slot<Event>, response_signal: Signal<Response>) {
    // spin up a slot to handle the events
    event_slot.start(move |event| {
        let response = process(event);
        if let Err(e) = response_signal.send(response) {
            eprintln!("Failed to send response: {:?}", e);
        }
    });
}

fn main() {
    let (response_signal, response_slot) = factory::create_signal_slot::<Response>(64);

    let dispatcher = Dispatcher::<Event>::new();

    // Register external_log handler on dispatcher
    dispatcher.register_slot("external_log", move |event| {
        let response = process(event);
        if let Err(e) = response_signal.send(response) {
            eprintln!("Failed to send response: {:?}", e);
        }
    });

    let app = UiApp::new(response_slot, dispatcher);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_titlebar_buttons_shown(true)
            .with_min_inner_size((750.0, 500.0))
            .with_resizable(true),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Dashboard with egui_mobius",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to run eframe UiApplication: {:?}", e);
    }
}
// src/app.rs

pub struct UiApp {
    dispatcher: Dispatcher<Event>,
    state: Value<AppState>,
}

impl UiApp {
    pub fn new(mut response_slot: Slot<Response>, dispatcher: Dispatcher<Event>) -> Self {
        let state = Value::new(AppState::default());

        // Register external_log handler on dispatcher
        let state_clone = state.clone();
        dispatcher.register_slot("external_log", move |event| {
            state_clone
                .lock()
                .unwrap()
                .log("external", format!("Dispatched: {:?}", event));
        });

        // Test message
        dispatcher.send(
            "external_log",
            Event::Custom("Hello from Dispatcher!".into()),
        );

        // Initialize the response listener exactly once
        let state_clone = state.clone();
        response_slot.start(move |response| {
            state_clone.lock().unwrap().handle_response(response);
        });

        Self { state, dispatcher }
    }

    pub fn log(&self, message: String) {
        let mut app_state = self.state.lock().unwrap();
        app_state.log("ui", message);
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            let mut app_state = self.state.lock().unwrap();

            //let mut app_state = self.state.get();

            egui::TopBottomPanel::bottom("log_panel")
                .resizable(true)
                .show(ctx, |ui| {
                    ui.heading("Logs");

                    ui.horizontal(|ui| {
                        ui.label("Source filter:");
                        for source in [&"ui", &"backend", &"external"] {
                            let source = source.to_string();
                            let selected = app_state.log_filters.contains(&source);
                            if ui.selectable_label(selected, &source).clicked() {
                                println!("selected: {}", selected);
                                match !selected {
                                    true => app_state.log_filters.push(source),
                                    false => app_state.log_filters.retain(|it| !it.eq(&source)),
                                }
                            }
                        }
                    });

                    ui.separator();

                    // Clear Logs Button
                    if ui.button("Clear Logger").clicked() {
                        app_state.logs.clear();
                    }

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut current_cluster_counter: Option<usize> = None;

                        for entry in app_state
                            .logs
                            .iter()
                            .rev()
                            .filter(|entry| app_state.log_filters.contains(&entry.source))
                        {
                            if entry.source == "backend"
                                && entry.message.starts_with("Counter updated to")
                            {
                                if let Some(new_counter) = entry
                                    .message
                                    .strip_prefix("Counter updated to ")
                                    .and_then(|v| v.parse::<usize>().ok())
                                {
                                    current_cluster_counter = Some(new_counter);
                                    ui.label(
                                        egui::RichText::new(format!(
                                            "\n**** Counter Event Cluster @ Counter == {}",
                                            new_counter
                                        ))
                                        .strong(),
                                    );
                                }
                            }

                            let colored = match entry.source.as_str() {
                                "external" => egui::RichText::new(entry.formatted())
                                    .color(egui::Color32::LIGHT_GREEN),
                                "ui" => egui::RichText::new(entry.formatted())
                                    .color(egui::Color32::YELLOW),
                                _ => egui::RichText::new(entry.formatted()),
                            };

                            ui.label(colored);
                        }

                        ui.add_space(10.0);
                    });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Your UI code here

            let counter = self.state.lock().unwrap().dashboard.counter;
            ui.label(format!("Counter: {}", counter));

            if ui.button("Increment").clicked() {
                self.log("clicked increment button".to_string());
                self.dispatcher
                    .send("external_log", Event::IncrementCounter);
                self.dispatcher.send(
                    "external_log",
                    Event::Custom("Clicked Increment button".into()),
                );
            }

            if ui.button("Reset").clicked() {
                self.log("clicked reset button".to_string());
                self.dispatcher.send("external_log", Event::ResetCounter);
                self.dispatcher
                    .send("external_log", Event::Custom("Clicked Reset button".into()));
            }

            if ui.button("Dispatch External").clicked() {
                self.dispatcher.send(
                    "external_log",
                    Event::Custom("Triggered from runtime UI button".into()),
                );
            }
        });
    }
}

#[derive(Default, Clone)]
pub struct DashboardState {
    pub counter: usize,
}

impl DashboardState {
    pub fn handle_response(&mut self, response: Response) {
        if let Response::CounterUpdated(value) = response {
            self.counter = value;
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    pub dashboard: DashboardState,
    pub logs: Vec<LogEntry>,
    pub log_filters: Vec<String>,
}

impl AppState {
    pub fn default() -> Self {
        Self {
            dashboard: DashboardState::default(),
            logs: Vec::new(),
            log_filters: vec!["ui".to_string()],
        }
    }

    pub fn log(&mut self, source: &str, message: String) {
        let entry = LogEntry {
            timestamp: Local::now(),
            source: source.to_string(),
            message,
        };

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("ui_session_log.txt")
        {
            let _ = writeln!(file, "{}", entry.formatted());
        }

        self.logs.push(entry);
        if self.logs.len() > 1000 {
            self.logs.drain(0..self.logs.len() - 1000);
        }
    }

    pub fn handle_response(&mut self, response: Response) {
        match response {
            Response::CounterUpdated(value) => {
                self.dashboard.counter = value;
                self.log("backend", format!("Counter updated to {}", value));
            }
            Response::Message(msg) => {
                self.log("backend", msg);
            }
        }
    }
}

// src/backend/processor.rs
lazy_static! {
    static ref COUNTER: Mutex<usize> = Mutex::new(0);
}

pub fn process(event: Event) -> Response {
    match event {
        Event::IncrementCounter => {
            let mut count = COUNTER.lock().unwrap();
            *count += 1;
            Response::CounterUpdated(*count)
        }
        Event::ResetCounter => {
            let mut count = COUNTER.lock().unwrap();
            *count = 0;
            Response::CounterUpdated(*count)
        }
        Event::Custom(message) => Response::Message(format!("processed message: {}", message)),
    }
}

pub fn view(
    state: &mut DashboardState,
    ui: &mut egui::Ui,
    signal: &Signal<Event>,
    _app_state: &mut AppState, // No direct logging here; backend handles logs
) {
    ui.label(format!("Counter: {}", state.counter));

    if ui.button("Increment").clicked() {
        let _ = signal.send(Event::IncrementCounter);
    }

    if ui.button("Reset").clicked() {
        let _ = signal.send(Event::ResetCounter);
    }
}

#[derive(Clone)]
pub struct LogEntry {
    timestamp: DateTime<Local>,
    source: String,
    message: String,
}

impl LogEntry {
    pub fn formatted(&self) -> String {
        format!(
            "[{}] [{}] {}",
            self.timestamp.format("%Y-%m-%d %H:%M:%S"),
            self.source,
            self.message
        )
        .to_string()
    }
}
