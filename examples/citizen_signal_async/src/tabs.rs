//! Tab definitions and the `TabViewer` bridge into `egui_dock`.

use eframe::egui;
use egui_citizen::{CitizenId, Dispatcher as CitizenDispatcher};

use crate::panels::{control::ControlPanel, logger::LoggerPanel, result::ResultPanel};
use crate::state::SharedState;

pub const CONTROL_ID: &str = "control";
pub const RESULT_ID:  &str = "result";
pub const LOGGER_ID:  &str = "logger";

#[derive(Clone, Copy)]
pub enum TabKind {
    Control,
    Result,
    Logger,
}

pub struct Tab {
    pub kind: TabKind,
}

impl Tab {
    pub fn new(kind: TabKind) -> Self { Self { kind } }

    pub fn title(&self) -> &'static str {
        match self.kind {
            TabKind::Control => "Control",
            TabKind::Result  => "Result",
            TabKind::Logger  => "Logger",
        }
    }

    pub fn citizen_id(&self) -> CitizenId {
        CitizenId::new(match self.kind {
            TabKind::Control => CONTROL_ID,
            TabKind::Result  => RESULT_ID,
            TabKind::Logger  => LOGGER_ID,
        })
    }
}

/// Bridge between `egui_dock` and the citizen layer.
///
/// `ui()` routes to each panel's render method. `on_tab_button`
/// forwards click events into `dispatcher.activate(...)` — the
/// canonical citizen hook so the dispatcher's queue stays accurate
/// even if the app doesn't currently drive behavior off activation.
pub struct TabViewer<'a> {
    pub state: &'a SharedState,
    pub dispatcher: &'a mut CitizenDispatcher,
    pub control: &'a mut ControlPanel,
    pub result:  &'a mut ResultPanel,
    pub logger:  &'a mut LoggerPanel,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.kind {
            TabKind::Control => self.control.show(ui, self.state),
            TabKind::Result  => self.result.show(ui, self.state),
            TabKind::Logger  => self.logger.show(ui, self.state),
        }
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.clicked() {
            self.dispatcher.activate(&tab.citizen_id());
        }
    }
}
