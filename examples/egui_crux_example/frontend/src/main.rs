use egui_mobius::factory;
use egui_mobius::types::Value;
use crate::ui_app::{UiApp, UiState};
use crate::ui_commands::{handle_command, UiCommand};

mod ui_app;
mod ui_commands;

mod backend_core;

pub fn main() {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let (signal, slot) = factory::create_signal_slot::<UiCommand>();
    
    let ui_state = Value::new(UiState::default());

    let core_service = Value::new(backend_core::CoreService::new(ui_state.clone()));

    let app = UiApp {
        ui_state: ui_state.clone(),
        command_sender: signal.sender.clone(),
    };

    // Define a handler function for the slot
    let handler = {
        let core_service = core_service.clone();
        let command_sender = signal.sender.clone();
        
        move |command: UiCommand| {
            handle_command(command, core_service.clone(), command_sender.clone());
        }
    };

    // Start the slot with the handler
    slot.start(handler);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size((650.0, 500.0)),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Mobius + Egui + Crux Example",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to run eframe: {:?}", e);
    }

}