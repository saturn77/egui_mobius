use eframe;
use backend::ProjectOverview;
use mobius_egui::types::{Enqueue, Value}; 
use mobius_egui::Signal;
use crate::ui_commands::UiCommand;

#[derive(Default)]
pub struct UiState {
    pub project_overview: Option<ProjectOverview>,
}

pub struct UiApp {
    pub ui_state: Value<UiState>,
    pub command_sender: Enqueue<UiCommand>,
}

impl eframe::App for UiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            // buttons
            ui.horizontal(|ui| {
                if ui.button("Create project").clicked() {
                    Signal!(self.command_sender, UiCommand::CreateProject);
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            
            let mut state = self.ui_state.lock().unwrap();
            if let Some(ref mut project_overview) = state.project_overview {
                ui.label("Name");
                ui.add(egui::TextEdit::singleline(&mut project_overview.name).interactive(false));
                
                ui.label("Description");
                ui.add(egui::TextEdit::singleline(&mut project_overview.description).interactive(false));
            } else {
                ui.spinner();
            }
        });
    }
}
