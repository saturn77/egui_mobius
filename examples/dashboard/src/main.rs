// src/main.rs
// mod app;
// mod backend;
// mod event;
// mod messaging;
// mod state;

use eframe::egui;
use egui_mobius::factory;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;
use egui_mobius::types::Value; 
 
use std::fmt::Debug; 
use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;
use lazy_static::lazy_static;
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
    let (event_signal, event_slot) = factory::create_signal_slot::<Event>(64);
    let (response_signal, response_slot) = factory::create_signal_slot::<Response>(64);

    let app = UiApp::new(event_signal.clone(), response_slot);

    // Spawn background_consumer_thread to handle Event -> Response processing
    std::thread::spawn(move || {
        run_backend(event_slot, response_signal);
    });

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
    state: Value<AppState>,
    event_signal: Signal<Event>,
}

impl UiApp {
    pub fn new(event_signal: Signal<Event>, mut response_slot: Slot<Response>) -> Self {
        let state = Value::new(AppState::new(event_signal.clone()));

        // Initialize the response listener exactly once
        let state_clone = state.clone();
        response_slot.start(move |response| {
            state_clone.lock().unwrap().handle_response(response);
        });

        Self { state, event_signal }
    }
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            let mut app_state = self.state.lock().unwrap();

            egui::SidePanel::left("log_panel").show(ctx, |ui| {
                ui.heading("Logs");

                ui.horizontal(|ui| {
                    ui.label("Source filter:");
                    for source in [&"ui", &"backend"] {
                        let selected = *source == app_state.log_filter;
                        if ui.selectable_label(selected, *source).clicked() {
                            app_state.log_filter = source.to_string();
                        }
                    }
                });

                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let filter_pattern = format!("[{}]", app_state.log_filter);
                    let logs_text: String = app_state
                        .logs
                        .iter()
                        .rev()
                        .filter(|entry| entry.contains(&filter_pattern))
                        .take(1000)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("\n");

                    let mut logs_text_mut = logs_text;

                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut logs_text_mut)
                            .font(egui::TextStyle::Monospace)
                            .desired_rows(30),
                    );
                });
            });
        }

        {
            let app_state = self.state.lock().unwrap();
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label(format!("Counter: {}", app_state.dashboard.counter));

                if ui.button("Increment").clicked() {
                    let _ = self.event_signal.send(Event::IncrementCounter);
                    let _ = self.event_signal.send(Event::Custom("Clicked Increment button".into()));
                }

                if ui.button("Reset").clicked() {
                    let _ = self.event_signal.send(Event::ResetCounter);
                    let _ = self.event_signal.send(Event::Custom("Clicked Reset button".into()));
                }
            });
        }
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
    pub event_signal: Signal<Event>,
    pub logs: Vec<String>,
    pub log_filter: String,
}

impl AppState {
    pub fn new(event_signal: Signal<Event>) -> Self {
        Self {
            dashboard: DashboardState::default(),
            event_signal,
            logs: Vec::new(),
            log_filter: "ui".into(),
        }
    }
    pub fn log(&mut self, source: &str, msg: String) {
        let timestamped = format!("[{}] [{}] {}", Local::now().format("%Y-%m-%d %H:%M:%S"), source, msg);
        self.logs.push(timestamped.clone());
        if self.logs.len() > 1000 {
            self.logs.drain(0..self.logs.len() - 1000);
        }

        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("ui_session_log.txt") {
            let _ = writeln!(file, "{}", timestamped);
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
            Response::Message(format!("[backend] Counter incremented to {}", *count))
        }
        Event::ResetCounter => {
            let mut count = COUNTER.lock().unwrap();
            *count = 0;
            Response::Message("[backend] Counter reset to 0".into())
        }
        Event::Custom(msg) => {
            Response::Message(format!("[ui] {}", msg))
        }
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

