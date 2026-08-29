//! Tab definitions and the `TabViewer` bridge into `egui_dock`.

use eframe::egui;
use egui_citizen::{CitizenId, Registry};

use crate::citizens::{console::ConsolePanel, logger::LoggerPanel, monitor::MonitorPanel};
use crate::state::SharedState;

pub const MONITOR_ID: &str = "monitor";
pub const CONSOLE_ID: &str = "console";
pub const LOGGER_ID: &str = "logger";

#[derive(Clone, Copy)]
pub enum TabKind {
    Monitor,
    Console,
    Logger,
}

pub struct Tab {
    pub kind: TabKind,
}

impl Tab {
    pub fn new(kind: TabKind) -> Self {
        Self { kind }
    }

    pub fn title(&self) -> &'static str {
        match self.kind {
            TabKind::Monitor => "Monitor",
            TabKind::Console => "Console",
            TabKind::Logger => "Logger",
        }
    }

    pub fn citizen_id(&self) -> CitizenId {
        CitizenId::new(match self.kind {
            TabKind::Monitor => MONITOR_ID,
            TabKind::Console => CONSOLE_ID,
            TabKind::Logger => LOGGER_ID,
        })
    }
}

/// Bridge between `egui_dock` and the citizen layer. `ui()` routes to
/// each citizen's render method; `on_tab_button` forwards clicks into
/// `registry.activate(...)` so the registry's queue stays accurate.
pub struct TabViewer<'a> {
    pub state: &'a SharedState,
    pub registry: &'a mut Registry,
    pub monitor: &'a mut MonitorPanel,
    pub console: &'a mut ConsolePanel,
    pub logger: &'a mut LoggerPanel,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn id(&mut self, tab: &mut Self::Tab) -> egui::Id {
        egui::Id::new(tab.title())
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.kind {
            TabKind::Monitor => self.monitor.show(ui, self.state),
            TabKind::Console => self.console.show(ui, self.state),
            TabKind::Logger => self.logger.show(ui, self.state),
        }
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.clicked() {
            self.registry.activate(&tab.citizen_id());
        }
    }
}
