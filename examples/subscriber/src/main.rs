//! Subscriber Example
//! 
//! This example demonstrates the enhanced subscription model for egui_mobius,
//! where multiple UI components can independently subscribe to and react to events.

use egui::Context;
use egui_mobius::dispatching::AsyncDispatcher;
use egui_mobius::factory;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;
use egui_mobius::types::Value;
use rand::Rng;
use std::collections::HashMap;

use chrono::{DateTime, Utc};

// Events that can be emitted
#[derive(Clone, Debug)]
pub enum Event {
    DataUpdate(String, f64),
    StartMonitoring,
    StopMonitoring,
    ClearHistory,
}

// Processed events that subscribers receive
#[derive(Clone, Debug)]
pub enum Processed {
    NewDataPoint {
        source: String,
        value: f64,
        timestamp: DateTime<Utc>,
    },
    MonitoringStarted,
    MonitoringStopped,
    HistoryCleared,
}

/// Trait for components that can be updated with processed events
pub trait Updatable<T> {
    fn update(&mut self, msg: T);
}

/// Chart view that displays historical data
#[derive(Default)]
pub struct ChartView {
    history: HashMap<String, Vec<(DateTime<Utc>, f64)>>,
}

impl ChartView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.push_id("chart_view", |ui| {
            ui.heading("Historical Data");
            
            egui::Grid::new("chart_grid_subscriber").show(ui, |ui| {
                ui.label("Source");
                ui.label("Points");
                ui.label("Latest");
                ui.end_row();

                for (source, data) in &self.history {
                    ui.label(source);
                    ui.label(format!("{}", data.len()));
                    if let Some((time, value)) = data.last() {
                        ui.label(format!("{:.2} @ {}", value, time.format("%H:%M:%S")));
                    }
                    ui.end_row();
                }
            });
        });
    }
}

impl Updatable<Processed> for ChartView {
    fn update(&mut self, msg: Processed) {
        match msg {
            Processed::NewDataPoint { source, value, timestamp } => {
                self.history.entry(source)
                    .or_default()
                    .push((timestamp, value));
            }
            Processed::HistoryCleared => {
                self.history.clear();
            }
            Processed::MonitoringStarted | Processed::MonitoringStopped => {}
        }
    }
}

/// Table view that shows current values
#[derive(Default)]
pub struct TableView {
    current_values: HashMap<String, (f64, DateTime<Utc>)>,
}

impl TableView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ui(&self, ui: &mut egui::Ui) {
        ui.push_id("table_view", |ui| {
            ui.heading("Current Values");
            
            egui::Grid::new("table_grid_subscriber").show(ui, |ui| {
                ui.label("Source");
                ui.label("Value");
                ui.label("Last Update");
                ui.end_row();

                for (source, (value, time)) in &self.current_values {
                    ui.label(source);
                    ui.label(format!("{:.2}", value));
                    ui.label(time.format("%H:%M:%S").to_string());
                    ui.end_row();
                }
            });
        });
    }
}

impl Updatable<Processed> for TableView {
    fn update(&mut self, msg: Processed) {
        match msg {
            Processed::NewDataPoint { source, value, timestamp } => {
                self.current_values.insert(source, (value, timestamp));
            }
            Processed::HistoryCleared => {
                self.current_values.clear();
            }
            Processed::MonitoringStarted | Processed::MonitoringStopped => {}
        }
    }
}

/// Main application window
pub struct MainWindow {
    chart_view: Value<ChartView>,
    table_view: Value<TableView>,
    monitoring: bool,
    event_signal: Signal<Event>,
    cancel_token: Option<tokio::sync::watch::Sender<bool>>,
}

impl MainWindow {
    pub fn new(event_signal: Signal<Event>, mut processed_slot: Slot<Processed>) -> Self {
        let chart_view = Value::new(ChartView::new());
        let table_view = Value::new(TableView::new());

        let chart_view_clone = chart_view.clone();
        let table_view_clone = table_view.clone();

        processed_slot.start(move |msg| {
            if let Ok(mut chart) = chart_view_clone.lock() {
                chart.update(msg.clone());
            }
            if let Ok(mut table) = table_view_clone.lock() {
                table.update(msg);
            }
        });

        Self {
            chart_view,
            table_view,
            monitoring: false,
            event_signal,
            cancel_token: None,
        }
    }

    fn start_monitoring(&mut self) {
        if !self.monitoring {
            self.monitoring = true;
            let event_signal = self.event_signal.clone();
            
            let (tx, mut rx) = tokio::sync::watch::channel(false);
            self.cancel_token = Some(tx);
            
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                loop {
                    tokio::select! {
                        Ok(_) = rx.changed() => {
                            if *rx.borrow() {
                                break;
                            }
                        }
                        _ = interval.tick() => {
                            // Simulate data updates
                            let sources = vec!["Sensor A", "Sensor B", "Sensor C"];
                            for source in sources {
                                let mut rng = rand::thread_rng();
                                let value = rng.gen_range(0.0..100.0);
                                match event_signal.send(Event::DataUpdate(source.to_string(), value)) {
                                    Ok(_) => continue,
                                    Err(_) => return,
                                }
                            }
                        }
                    }
                }
            });
        }
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let text = if self.monitoring { "Stop Monitoring" } else { "Start Monitoring" };
                let mut button = egui::Button::new(text)
                    .min_size(egui::vec2(120.0, 30.0));
                
                if self.monitoring {
                    button = button.stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(50, 200, 50)));
                } else {
                    button = button.stroke(egui::Stroke::new(2.0, egui::Color32::from_rgb(200, 50, 50)));
                }
                
                if ui.add(button).clicked() {
                    if self.monitoring {
                        self.monitoring = false;
                        if let Some(token) = self.cancel_token.take() {
                            let _ = token.send(true);
                        }
                        if let Err(e) = self.event_signal.send(Event::StopMonitoring) {
                            eprintln!("Failed to send stop monitoring event: {}", e);
                        }
                    } else {
                        self.start_monitoring();
                        if let Err(e) = self.event_signal.send(Event::StartMonitoring) {
                            eprintln!("Failed to send start monitoring event: {}", e);
                        }
                    }
                }

                if ui.button("Clear History").clicked() {
                    if let Err(e) = self.event_signal.send(Event::ClearHistory) {
                        eprintln!("Failed to send clear history event: {}", e);
                    }
                }
            });

            ui.add_space(20.0);

            // Display both views side by side using columns for better layout
            ui.columns(2, |columns| {
                // Left column: Table view
                columns[0].with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("table_scroll")
                        .min_scrolled_height(300.0)
                        .show(ui, |ui| {
                            if let Ok(view) = self.table_view.lock() {
                                view.ui(ui);
                            }
                        });
                });

                // Right column: Chart view
                columns[1].with_layout(egui::Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt("chart_scroll")
                        .min_scrolled_height(300.0)
                        .show(ui, |ui| {
                            if let Ok(view) = self.chart_view.lock() {
                                view.ui(ui);
                            }
                        });
                });
            });
        });

        // Request continuous updates
        ctx.request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create signal-slot pairs
    let (event_signal, event_slot) = factory::create_signal_slot::<Event>();
    let (processed_signal, processed_slot) = factory::create_signal_slot::<Processed>();

    // Create the main window
    let app = MainWindow::new(event_signal.clone(), processed_slot);

    // Create the async runtime and dispatcher
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let _guard = runtime.enter();

    let dispatcher = AsyncDispatcher::<Event, Processed>::new();
    
    // Attach event processor
    dispatcher.attach_async(event_slot, processed_signal, |event| async move {
        match event {
            Event::DataUpdate(source, value) => {
                Processed::NewDataPoint {
                    source,
                    value,
                    timestamp: chrono::Utc::now(),
                }
            }
            Event::StartMonitoring => {
                Processed::MonitoringStarted
            }
            Event::StopMonitoring => {
                Processed::MonitoringStopped
            }
            Event::ClearHistory => {
                Processed::HistoryCleared
            }
        }
    });

    // Run the app
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size((800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Subscriber Example",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
