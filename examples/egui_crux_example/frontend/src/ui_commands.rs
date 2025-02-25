use log::debug;
use backend::{CreateProjectArgs, Event, ViewRequest};
use egui_mobius::types::{Enqueue, Value};
use crate::backend_core::CoreService;

#[derive(Debug, Clone)]
pub enum UiCommand {
    #[allow(dead_code)]
    None,
    
    // project commands
    CreateProject,
    
    // views
    ProjectOverview,
}

pub fn handle_command(
    command: UiCommand,
    core_service: Value<CoreService>,
    command_sender: Enqueue<UiCommand>,
) {
    match command {
        UiCommand::None => {}
        UiCommand::CreateProject => {
            debug!("Creating project...");
            let mut core_service = core_service.lock().unwrap();

            // TODO interact with crux backend here
            
            let args = CreateProjectArgs {
                name: "example name".to_string(),
                description: "example description".to_string(),
            };
            core_service.update(Event::CreateProject(args), command_sender.clone());
            
            command_sender.send(UiCommand::ProjectOverview).expect("Failed to send command to ui");
        }
        UiCommand::ProjectOverview => {
            let mut core_service = core_service.lock().unwrap();
            core_service.update(Event::RequestView(ViewRequest::ProjectOverview), command_sender.clone());
        }
    }
}