pub use crux_core::App;
pub use crux_core::{Core, Request};

use crux_core::{render, render::Render, Command};
use serde::{Deserialize, Serialize};
use project::Project;

use crate::view_renderer::ViewRenderer;

pub mod view_renderer;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateProjectArgs {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMemberArgs {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ViewRequest {
    ProjectOverview
}


#[derive(Debug, Serialize, Deserialize)]
pub enum Event {
    CreateProject(CreateProjectArgs),

    SetName(String),
    AddMember(AddMemberArgs),

    RequestView(ViewRequest),
}

#[derive(Debug)]
pub struct ProjectState {
    project: Project,
    modified: bool,
}

#[derive(Default, Debug)]
pub struct Model {
    state: Option<ProjectState>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct OperationViewModel {
    pub modified: bool,
    pub error: Option<String>,
}

#[derive(crux_core::macros::Effect)]
#[allow(unused)]
pub struct Capabilities {
    render: Render<Event>,
    view: ViewRenderer<Event>,
}


#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ProjectOverview {
    pub name: String,
    pub description: String,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Member {
    pub name: String,
}


#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum View {
    ProjectOverview(ProjectOverview),
    Members(Vec<Member>),
    Member(Member),
}


#[derive(Default)]
pub struct ProjectApp;

impl App for ProjectApp {
    type Event = Event;
    type Model = Model;
    type ViewModel = OperationViewModel;
    type Capabilities = Capabilities;
    type Effect = Effect;

    fn update(
        &self,
        _event: Self::Event,
        _model: &mut Self::Model,
        _caps: &Self::Capabilities,
    ) -> Command<Effect, Event> {
        // we no longer use the capabilities directly, but they are passed in
        // until the migration to managed effects with `Command` is complete
        // (at which point the capabilities will be removed from the `update`
        // signature). Until then we delegate to our own `update` method so that
        // we can test the app without needing to use AppTester.
        self.update(_event, _model)
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        OperationViewModel {
            modified: model.state.as_ref().map_or(false, |state|state.modified),
            error: model.error.clone(),
        }
    }
}

impl ProjectApp {
    // note: this function can be moved into the `App` trait implementation, above,
    // once the `App` trait has been updated (as the final part of the migration
    // to managed effects with `Command`).
    fn update(&self, _event: Event, _model: &mut Model) -> Command<Effect, Event> {

        match _event {
            Event::CreateProject(args) => {
                _model.state = Some(ProjectState { project: Project::new(args.name, args.description), modified: true });
                _model.error = None;

                render::render()
            }
            Event::SetName(name) => {
                if let Some(state) = &mut _model.state {
                    state.project.set_name(name);
                } else {
                    _model.error = Some("Project required".to_string());
                }
                render::render()
            }
            Event::AddMember(args) => {
                if let Some(state) = &mut _model.state {
                    state.project.add_member(args.name);
                } else {
                    _model.error = Some("Project required".to_string());
                }
                render::render()
            }

            Event::RequestView(view) => {
                match view {
                    ViewRequest::ProjectOverview => {
                        if let Some(state) = &mut _model.state {
                            view_renderer::view(
                                View::ProjectOverview(
                                    ProjectOverview {
                                        name: state.project.name(),
                                        description: state.project.description()
                                    }
                                )
                            )
                        } else {
                            _model.error = Some("Project required".to_string());
                            render::render()
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_project() {
        let hello = ProjectApp::default();
        let mut model = Model { state: None, error: None };

        let args = CreateProjectArgs {
            name: "test name".to_string(),
            description: "test description".to_string(),
        };

        // Call 'update' and request effects
        let mut cmd = hello.update(Event::CreateProject(args), &mut model);

        // Check update asked us to `Render`
        cmd.expect_one_effect().expect_render();

        // Make sure the view matches our expectations
        let view = &hello.view(&model);
        assert_eq!(view.error, None);
        assert_eq!(view.modified, true);
    }
}

