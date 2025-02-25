use std::sync::Arc;
use backend::{Effect, Event, ProjectApp, View};
use log::debug;
use backend::view_renderer::ViewRendererOperation;
use egui_mobius::types::{Enqueue, Value};
use crate::ui_app::UiState;
use crate::ui_commands::UiCommand;

type Core = Arc<backend::Core<ProjectApp>>;

pub struct CoreService {
    core: Core,
    ui_state: Value<UiState>,
}

impl CoreService {
    pub fn new(ui_state: Value<UiState>) -> Self {
        Self {
            core: Arc::new(backend::Core::new()),
            ui_state,
        }
    }

    pub fn update(&mut self, event: Event, sender: Enqueue<UiCommand>) {
        debug!("event: {:?}", event);

        for effect in self.core.process_event(event) {
            Self::process_effect(&self.core, effect, sender.clone(), self.ui_state.clone());
        }
    }

    pub fn process_effect(core: &Core, effect: Effect, _sender: Enqueue<UiCommand>, ui_state: Value<UiState>) {
        debug!("effect: {:?}", effect);

        match effect {
            Effect::Render(_) => {
                let view = core.view();
                if view.modified == true {
                    debug!("modified");
                }
            }
            Effect::ViewRenderer(request) => {
                let ViewRendererOperation::View {
                    view,
                } = request.operation;

                let mut ui_state = ui_state.lock().unwrap();
                match view {
                    View::ProjectOverview(project_overview) => {
                        ui_state.project_overview = Some(project_overview)
                    }
                    View::Members(_) => {}
                    View::Member(_) => {}
                }

            }
        }
    }
}