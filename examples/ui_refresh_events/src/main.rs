//-------------------------------------------------------------------------
// Filename : examples/ui_refresh_events/src/main.rs
// Project  : egui_mobius
// Created  : 05 Mar 2025, James B <atlantix-eda@proton.me>
//-------------------------------------------------------------------------
// Description: 
// This example extends the ui_refresh example to include dynamic event types.
// It demonstrates how to use egui_mobius to optimize UI updates by using
// signals and slots to manage message passing between threads. This is related
// to the dispatcher_signals_slots example, but with a focus on UI updates.
//
// The example shows how to create a dynamic event type that can be used to
// send different types of messages to the UI thread. The UI thread updates
// the UI based on the received messages. The producer thread (effectivelye
// the code where "ui_app.rs" would be, sends messages across a single 
// Signal/Slot pair, which then the consumer thread consumes and then 
// updates the UI. The UI thread only repaints when a new message is received.
//
//-------------------------------------------------------------------------
use eframe::egui;
use egui_mobius::factory;
use egui_mobius::signals::Signal;
use egui_mobius::slot::Slot;
use egui_mobius::types::{Value, Edge}; 
use std::collections::VecDeque;
use log::{info, warn};
use std::fmt::Debug;
//----------------------------------------------------------------------------
// **Event Type**
//----------------------------------------------------------------------------
// The event type is an enum that defines the different types of events that
// can be sent **from** the UI. The event type is used to send messages to the 
// backend_consumer_thread, which then processes the messages and updates the UI.
//----------------------------------------------------------------------------
#[derive(Debug, Clone)]
enum EventType {
    Foo { id: usize, message: String },
    Bar { id: usize, message: String },
    Slider(usize),
    Combo { _id: usize, message: String },    
    ApplicationCommand(String),
}

#[derive(Debug, Clone)]
enum ProcessedType {
    Foo    { _id: usize, message: String   },
    Bar    { _id: usize, message: String   },
    Slider { message: String              },
    _Combo  { _id: usize, message: String  },    
    _ApplicationCommand { message : String },
}

//----------------------------------------------------------------------------
// **UiApp Struct**
//----------------------------------------------------------------------------
// The UiApp struct containes the shared data or "state" that is used to
// update the Ui.
//----------------------------------------------------------------------------
struct UiApp {
    logger_text         : Value<String>,
    signal_to_backend   : Signal<EventType>,  
    slot_on_uiapp       : Slot<ProcessedType>,  
    update_needed       : Value<bool>,
    slider_value        : Value<Edge<usize>>,
    combo_value         : Value<Edge<String>>,
}
//----------------------------------------------------------------------------
// **UiApp Implementation*
//----------------------------------------------------------------------------
// The Ui App implementation is where methods, such as new, are defined.
// The new method is used to create a new instance of the UiApp struct.
// The eframe::App trait is implemented for the UiApp struct.
//----------------------------------------------------------------------------
impl UiApp {
    fn new(
        signal_to_backend    : Signal<EventType>,  
        slot_on_uiapp        : Slot<ProcessedType>,  
        update_needed        : Value<bool>,
    ) -> Self {
        Self {
            logger_text: Value::new("**** Welcome to egui_mobius ui_refresh_events example ....\n\n".to_string()),
            signal_to_backend,
            slot_on_uiapp,
            update_needed,
            slider_value: Value::new(Edge::new(0)),
            combo_value: Value::new(Edge::new("Egui_with_egui_mobius".to_string())),
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
        let mut should_repaint = false;

        self.slot_on_uiapp.start({
            let logger_text_clone = self.logger_text.clone();
            let update_needed_clone = self.update_needed.clone();
            move |event: ProcessedType| {
                let mut logger: egui_mobius::types::ValueGuard<'_, String> = logger_text_clone.lock().unwrap();
                let log_msg: String = match event {
                    ProcessedType::Foo    { _id: _, message  } => format!("Ui Received:  {}", message),
                    ProcessedType::Bar    { _id: _, message  } => format!("Ui Received:  {}", message),
                    ProcessedType::Slider { message: value      } => format!("Ui Received:  {}", value),
                    ProcessedType::_Combo  { _id, message } => {
                        format!("Ui Received: [{}]: {}", _id, message)
                    },
                    ProcessedType::_ApplicationCommand { message } => {
                        match message.as_str() {
                            "Backend cleared the logger" => {
                                logger.clear();
                                "".to_string()
                            }
                            // can we use regex or starts_with to match the command? Yes ! 
                            
                            msg if msg.starts_with("Backend found the os") => {
                                format!("Ui Received:  {}", message)
                            },
                            msg if msg.starts_with("Backend found version info") => {
                                format!("Ui Received:  {}", message)
                            },
                            _ => "Unknown command".to_string(),
                        }
                    }
                    
                };
                logger.push_str(&format!("{}\n", log_msg));
                *update_needed_clone.lock().unwrap() = true;
            }
        });



        egui::CentralPanel::default().show(ctx, |ui| {
            
            ui.label("Buttons");  
            ui.add_space(5.0);

            // add back in the Foo and Bar buttons for testing
            ui.horizontal(|ui| {
                if ui.button("Foo").clicked() {
                    self.signal_to_backend.send(EventType::Foo { id: 1, message: "Foo Button Clicked".to_string() }).unwrap();
                }
                if ui.button("Bar").clicked() {
                    self.signal_to_backend.send(EventType::Bar { id: 2, message: "Bar Button Clicked".to_string() }).unwrap();
                }
            });
            ui.add_space(10.0);

            ui.separator();
            ui.label("Slider");  
            ui.add_space(5.0);
            
            
            let mut slider_value = self.slider_value.lock().unwrap();
            ui.add(egui::Slider::new(&mut slider_value.values[0], 0..=100).text("Value"));
            ui.add_space(10.0);

            if !slider_value.are_values_equal() {
                let value = slider_value.values[0];
                self.signal_to_backend.send(EventType::Slider(value)).unwrap();
                slider_value.add_value(value);
                should_repaint = true;
            }


            ui.separator();
            
            ui.label("Combo Box");  
            ui.add_space(5.0);
            
            let combo_items = vec![
                "Egui_with_egui_mobius".to_string(),
                "Egui_with_mobius_crux".to_string(),
                "Cushy".to_string(),
                "Dioxus".to_string(),
                "Tauri".to_string(),
                "Slint".to_string(),
            ];
            
            // Lock `combo_value`
            // Lock `combo_value`
            let mut combo_value = self.combo_value.lock().unwrap();
            let previous_value = combo_value.values[0].clone();
            
            egui::ComboBox::from_label("Select an option")
                .selected_text(&previous_value)
                .show_ui(ui, |ui| {
                    for item in &combo_items {
                        ui.selectable_value(&mut combo_value.values[0], item.clone(), item);
                    }
                });
            ui.add_space(10.0);
            
            // Send the combo box event if the value has changed
            if !combo_value.are_values_equal() {
                let new_value = combo_value.values[0].clone();
                self.signal_to_backend.send(EventType::Combo { _id: 1, message: new_value.clone() }).unwrap();
                combo_value.add_value(new_value);
                should_repaint = true;
            }

            ui.separator();
            
            ui.label("Application Commands");  
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                if ui.button("Clear Logger").clicked() {
                    self.signal_to_backend.send(EventType::ApplicationCommand("Clear Logger".to_string())).unwrap();
                }
                if ui.button("OS Info").clicked() {
                    self.signal_to_backend.send(EventType::ApplicationCommand("OS Info".to_string())).unwrap();
                }
                if ui.button("Version Info").clicked() {
                    self.signal_to_backend.send(EventType::ApplicationCommand("Version Info".to_string())).unwrap();
                }
                if ui.button("Shutdown").clicked() {
                    self.signal_to_backend.send(EventType::ApplicationCommand("Shutdown".to_string())).unwrap();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.label("Received Messages:");
            ui.add_space(10.0);

            egui::ScrollArea::vertical()
                .id_salt("terminal_scroller")
                .stick_to_bottom(true)
                .max_height(400.0)
                .min_scrolled_height(400.0)
                .show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut *self.logger_text.lock().unwrap())
                        .min_size(egui::vec2(400.0, 400.0))
                        .desired_width(f32::INFINITY)
                        .text_color(egui::Color32::GREEN)
                        .id(egui::Id::new("terminal"))
                        .desired_rows(20)
                        .font(egui::TextStyle::Monospace)
                        .interactive(true));
                });
        });

        let mut update_needed = self.update_needed.lock().unwrap();
        if *update_needed || should_repaint {
            ctx.request_repaint();
            *update_needed = false;
        }
    }
}

// Macro to handle logging and sending processed messages
macro_rules! process_event {
    ($queue:expr, $logger:expr, $slot:expr, $event:expr, $log_msg:expr, $processed_msg:expr) => {
        $queue.push_back($log_msg.clone());
        $logger.push_str(&format!("{}\n", $log_msg));
        $slot.send($processed_msg).unwrap();
    };
}

//-------------------------------------------------------------------------
// **Backend Thread**
//-------------------------------------------------------------------------
// Note that Slot's create their own thread, so this function is effectively
// running in a separate thread. Also note that the egui_mobius type Value
// is used to share data between threads.
//-------------------------------------------------------------------------
fn backend_consumer_thread(
    logger_text         : Value<String>, 
    messages            : Value<VecDeque<String>>, 
    update_needed       : Value<bool>, 
    mut slot            : Slot<EventType>,         // incoming from UiApp
    slot_on_uiapp       : Signal<ProcessedType>,   // outgoing to UiApp
) {
    slot.start({
        let messages_clone = Value::clone(&messages);
        let update_needed_clone = Value::clone(&update_needed);
        let logger_text_clone = Value::clone(&logger_text);
        move |event| {
            let mut queue = messages_clone.lock().unwrap();
            let mut logger = logger_text_clone.lock().unwrap();
            
            let _log_msg = match &event {
                EventType::Foo { id, message } => {
                    let log_msg = format!("Backend processed Foo event [{}]: {}", id, message);
                    let processed_msg = ProcessedType::Foo { _id: *id, message: log_msg.clone() };
                    process_event!(queue, logger, slot_on_uiapp, event, log_msg, processed_msg);
                    return
                },
                EventType::Bar { id, message } => {
                    let log_msg = format!("Backend processed Bar event [{}]: {}", id, message);
                    let processed_msg = ProcessedType::Bar { _id: *id, message: log_msg.clone() };
                    process_event!(queue, logger, slot_on_uiapp, event, log_msg, processed_msg);
                    return
                },
                EventType::Slider(value) => {
                    let log_msg = format!("Backend processed Slider value: {}", value);
                    let processed_msg = ProcessedType::Slider{ message: log_msg.clone() };
                    process_event!(queue, logger, slot_on_uiapp, event, log_msg, processed_msg);
                    return
                },
                EventType::Combo { message: msg, _id } => {
                    let log_msg = format!("Backend processed Combo selection: {}", msg);
                    let processed_msg = ProcessedType::_Combo { _id: *_id, message: log_msg.clone() };
                    process_event!(queue, logger, slot_on_uiapp, event, log_msg, processed_msg);
                    return
                },

                EventType::ApplicationCommand(cmd) => {
                    let _log_msg = match cmd.as_str() {
                        "Clear Logger" => {
                            warn!("Clearing Logger...");
                            let log_msg = "Backend cleared the logger".to_string();
                            let processed_msg = ProcessedType::_ApplicationCommand { message: log_msg.clone() };
                            process_event!(queue, logger, slot_on_uiapp, event, log_msg, processed_msg);
                            return
                        }
                        "OS Info" => {
                            info!("Displaying OS Info...");
                            let log_msg = format!("Backend found the os OS: {} | ARCH: {}", std::env::consts::OS, std::env::consts::ARCH);
                            process_event!(queue, logger, slot_on_uiapp, event, log_msg, ProcessedType::_ApplicationCommand { message: log_msg.clone() });
                            return
                        },
                        "Version Info" => {
                            info!("Displaying Version Info...");
                            let log_msg = "Backend found version info: 1.0.0".to_string();
                            process_event!(queue, logger, slot_on_uiapp, event, log_msg, ProcessedType::_ApplicationCommand { message: log_msg.clone() });
                            return
                        },
                        "Shutdown" => {
                            warn!("Shutting down...");
                            queue.push_back("Shutting down...".to_string());
                            logger.push_str("Shutting down...\n");
                            let log_msg = "Shutting down...".to_string();
                            process_event!(queue, logger, slot_on_uiapp, event, log_msg, ProcessedType::_ApplicationCommand { message: log_msg.clone() });
                            
                            std::thread::sleep(std::time::Duration::from_millis(1500));
                            std::process::exit(0);
                        }
                        _ => "Unknown command".to_string(),
                    };
                    // let processed_msg = ProcessedType::_ApplicationCommand { message: log_msg.clone() };
                    // process_event!(queue, logger, slot_on_uiapp, event, log_msg, processed_msg);
                }

                
            };
            
            *update_needed_clone.lock().unwrap() = true;
        }
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
    env_logger::init();

    let messages = Value::new(VecDeque::new());
    let update_needed = Value::new(false);
    let logger_text = Value::new("Welcome to egui_mobius ui_refresh_events example ....\n".to_string());

    let (signal_to_backend, slot_to_backend) = factory::create_signal_slot::<EventType>(1);
    let (slot_on_uiapp, slot_from_backend) = factory::create_signal_slot::<ProcessedType>(1);

    backend_consumer_thread(
        Value::clone(&logger_text),
        Value::clone(&messages),
        Value::clone(&update_needed),
        slot_to_backend,
        slot_on_uiapp.clone(),
    );


    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_titlebar_buttons_shown(true)
            .with_min_inner_size((650.0, 700.0))
            .with_resizable(true)
            .with_max_inner_size((650.0, 700.0)),
        ..Default::default()
    };


    if let Err(e) = eframe::run_native(
        "egui_mobius - UI Refresh Events Example",
        options,
        Box::new(|_cc| Ok(Box::new(UiApp::new(
            signal_to_backend,
            slot_from_backend,
            update_needed,
        )))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }
}
