//-------------------------------------------------------------------------
// Filename : examples/realtime_plot/src/main.rs
// Project  : egui_mobius
// Created  : 09 Mar 2025, James B <atlantix-eda@proton.me>
//-------------------------------------------------------------------------
// Description: 
// This example shows how to create a real-time plotting application in the
// context of temperature plotting. The application has two threads: a producer
// thread that sends temperature data every second, and a consumer thread that
// receives the data and updates the UI. The UI thread updates the UI based on
// the received messages. The producer thread sends messages across a single
// Signal/Slot pair, which the consumer thread consumes and then updates the UI.
//
// A key component of egui_mobius is it's data types, such as Value and Edge.
// These data types are used to facilitate the communication between the
// producer and consumer threads. The Value type is used to store the temperature
// data. Additionally the context of the UiApp data is called "Fabric" in this
// example. This is to avoid confusion with the UI context. The Fabric data
// contains the temperature data and the history of the temperature data.
//
//-------------------------------------------------------------------------
use eframe::egui;
use egui_mobius::{factory, signals::Signal, slot::Slot};
use egui_plot::{Line, Plot, PlotPoints, Legend};
use egui_mobius::types::Value;
use std::thread;
use std::time::Duration;

// Define some global constants
const MAX_HISTORY_LEN: usize = 300;

//----------------------------------------------------------------------------
// **Event Type**
//----------------------------------------------------------------------------
// The event type is an enum that defines the different types of events that
// can be sent **from** the UI. The event type is used to send messages to the 
// backend_consumer_thread, which then processes the messages and updates the UI.
// In this case, the event type is used to send the temperature data from the
// producer thread to the consumer thread.
//----------------------------------------------------------------------------
#[derive(Debug, Clone)]
enum Event {
    DataUpdated { inlet: f64, exhaust: f64, ambient: f64 },
}

//----------------------------------------------------------------------------
// **UiApp Struct**
//----------------------------------------------------------------------------
// The UiApp struct containes the shared data or "state" that is used to
// update the Ui.
//----------------------------------------------------------------------------
#[allow(dead_code)]
struct UiApp {
    fabric_data : Fabric,
    ui_signal   : Signal<Event>,
    ui_slot     : Slot<Event>,
}
struct Fabric {
    inlet_temp      : Value<f64>,
    exhaust_temp    : Value<f64>,
    ambient_temp    : Value<f64>,
    inlet_history   : Value<Vec<f64>>,
    exhaust_history : Value<Vec<f64>>,
    ambient_history : Value<Vec<f64>>,
    y_bounds        : Value<(f64, f64)>,
}
//----------------------------------------------------------------------------
// **UiApp Implementation*
//----------------------------------------------------------------------------
// The Ui App implementation is where methods, such as new, are defined.
// The new method is used to create a new instance of the UiApp struct.
// The eframe::App trait is implemented for the UiApp struct.
//----------------------------------------------------------------------------
impl UiApp {
    fn new(ui_signal: Signal<Event>, ui_slot: Slot<Event>) -> Self {
        Self {
            fabric_data: Fabric {
                inlet_temp: Value::new(100.0),
                exhaust_temp: Value::new(200.0),
                ambient_temp: Value::new(25.0),
                inlet_history: Value::new(vec![100.0; MAX_HISTORY_LEN]),
                exhaust_history: Value::new(vec![200.0; MAX_HISTORY_LEN]),
                ambient_history: Value::new(vec![25.0; MAX_HISTORY_LEN]),
                y_bounds: Value::new((0.0, 500.0)), // Default y-axis range
            },
            ui_signal,
            ui_slot,
        }
    }
}
//----------------------------------------------------------------------------
// **UiApp eframe::App **
//----------------------------------------------------------------------------
// The eframe::App trait is implemented for the UiApp struct. The update
// method is used to update the UI based on the shared data within the UiApp
// struct. The update method is where the UI is updated based on the shared
// data. The shared data is updated by the background_consumer_thread, which
// sends messages via a signal/slot pair to the UI thread. The UI thread then
// updates the UI based on the received messages.
//----------------------------------------------------------------------------
impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let fabric_data = &self.fabric_data;
        let mut inlet_temp = fabric_data.inlet_temp.lock().unwrap();
        let mut exhaust_temp = fabric_data.exhaust_temp.lock().unwrap();
        let mut ambient_temp = fabric_data.ambient_temp.lock().unwrap();
        let inlet_history = fabric_data.inlet_history.lock().unwrap();
        let exhaust_history = fabric_data.exhaust_history.lock().unwrap();
        let ambient_history = fabric_data.ambient_history.lock().unwrap();
        let y_bounds = fabric_data.y_bounds.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Real-Time Temperature Data");

            // Sliders for temperature input
            ui.horizontal(|ui| {
                ui.label("Inlet Temperature (°C):");
                if ui.add(egui::Slider::new(&mut *inlet_temp, 0.0..=500.0)).changed() {
                    let _ = self.ui_signal.send(Event::DataUpdated { inlet: *inlet_temp, exhaust: *exhaust_temp, ambient: *ambient_temp });
                }
            });

            ui.horizontal(|ui| {
                ui.label("Exhaust Temperature (°C):");
                if ui.add(egui::Slider::new(&mut *exhaust_temp, 0.0..=500.0)).changed() {
                    let _ = self.ui_signal.send(Event::DataUpdated { inlet: *inlet_temp, exhaust: *exhaust_temp, ambient: *ambient_temp });
                }
            });

            ui.horizontal(|ui| {
                ui.label("Ambient Temperature (°C):");
                if ui.add(egui::Slider::new(&mut *ambient_temp, 0.0..=100.0)).changed() {
                    let _ = self.ui_signal.send(Event::DataUpdated { inlet: *inlet_temp, exhaust: *exhaust_temp, ambient: *ambient_temp });
                }
            });

            ui.separator();

            // Temperature plot with legend
            ui.label("Temperature History");
            Plot::new("temp_plot")
                .view_aspect(2.0)
                .include_y(y_bounds.0)
                .include_y(y_bounds.1)
                .legend(Legend::default())
                .show(ui, |plot_ui| {
                    let inlet_points: PlotPoints = inlet_history.iter().enumerate().map(|(i, &y)| [i as f64, y]).collect();
                    let exhaust_points: PlotPoints = exhaust_history.iter().enumerate().map(|(i, &y)| [i as f64, y]).collect();
                    let ambient_points: PlotPoints = ambient_history.iter().enumerate().map(|(i, &y)| [i as f64, y]).collect();

                    plot_ui.line(Line::new(inlet_points).name("Inlet Temp (°C)").color(egui::Color32::RED));
                    plot_ui.line(Line::new(exhaust_points).name("Exhaust Temp (°C)").color(egui::Color32::BLUE));
                    plot_ui.line(Line::new(ambient_points).name("Ambient Temp (°C)").color(egui::Color32::GREEN));
                });
        });

        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

//-------------------------------------------------------------------------
// **Backend Processes - Two Threads**
//-------------------------------------------------------------------------
// This illustrates multiple background threads. The producer thread sends
// temperature data every second, and the consumer thread receives the data
// and updates the UI.
//-------------------------------------------------------------------------
macro_rules! append_and_maintain_fifo {
    ($history:expr, $new_value:expr, $max_len:expr) => {
        $history.push($new_value);
        if $history.len() > $max_len {
            $history.remove(0);
        }
    };
}
// **Producer Thread: Sends new temperature data every second**
fn producer_thread(signal: Signal<Event>, fabric_data: &Fabric) {
    let inlet = fabric_data.inlet_temp.clone();
    let exhaust = fabric_data.exhaust_temp.clone();
    let ambient = fabric_data.ambient_temp.clone();

    thread::spawn(move || {
        loop {
            let inlet_val = *inlet.lock().unwrap();
            let exhaust_val = *exhaust.lock().unwrap();
            let ambient_val = *ambient.lock().unwrap();

            if signal.send(Event::DataUpdated { inlet: inlet_val, exhaust: exhaust_val, ambient: ambient_val }).is_err() {
                eprintln!("Failed to send data update from producer.");
            }

            thread::sleep(Duration::from_secs(1));
        }
    });
}
// **Consumer Thread: Receives updates and maintains FIFO buffer**
fn consumer_thread(mut slot: Slot<Event>, fabric_data: &Fabric) {
    let inlet = fabric_data.inlet_temp.clone();
    let exhaust = fabric_data.exhaust_temp.clone();
    let ambient = fabric_data.ambient_temp.clone();
    let inlet_history = fabric_data.inlet_history.clone();
    let exhaust_history = fabric_data.exhaust_history.clone();
    let ambient_history = fabric_data.ambient_history.clone();


        slot.start(move |event| {
            let Event::DataUpdated { inlet: new_inlet, exhaust: new_exhaust, ambient: new_ambient } = event;

            // Store latest values persistently
            *inlet.lock().unwrap() = new_inlet;
            *exhaust.lock().unwrap() = new_exhaust;
            *ambient.lock().unwrap() = new_ambient;

            let mut inlet_history = inlet_history.lock().unwrap();
            let mut exhaust_history = exhaust_history.lock().unwrap();
            let mut ambient_history = ambient_history.lock().unwrap();

            // Append new values and maintain FIFO buffer using macro
            append_and_maintain_fifo!(inlet_history, new_inlet, 300);
            append_and_maintain_fifo!(exhaust_history, new_exhaust, 300);
            append_and_maintain_fifo!(ambient_history, new_ambient, 300);
        });
    
}



//-------------------------------------------------------------------------
// **Main Function**
//-------------------------------------------------------------------------
// Note that the general design pattern for using egui_mobius is to declare
// the shared data as Value types, and then pass them to the
// application-specific struct that implements eframe::App. This struct
// will then be used to create the UI.
//-------------------------------------------------------------------------
// Overall the main function is where the shared data is created, the 
// signal/slot pair is created, and the consumer thread is started. The 
// UI is then initialized and run using eframe.
// The compactness of the code is due to the use of egui_mobius to manage
// the shared data and the signal/slot pair.
//-------------------------------------------------------------------------
fn main() {
    let (ui_signal, ui_slot) = factory::create_signal_slot();
    let app = UiApp::new(ui_signal.clone(), ui_slot.clone());

    producer_thread(ui_signal.clone(), &app.fabric_data);
    consumer_thread(ui_slot, &app.fabric_data);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_titlebar_buttons_shown(true)
            .with_min_inner_size((750.0, 500.0))
            .with_resizable(true),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Real-Time Plot Example with egui_mobius",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to run eframe UiApplication: {:?}", e);
    }
}
