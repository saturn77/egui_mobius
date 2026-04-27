//! Tab definitions and the `TabViewer` bridge into `egui_dock`.

use eframe::egui;
use egui_citizen::{CitizenId, Dispatcher};

use crate::panels::{plot::PlotPanel, settings::SettingsPanel, terminal::TerminalPanel};
use crate::state::SharedState;

pub const PLOT_ID:     &str = "plot";
pub const SETTINGS_ID: &str = "settings";
pub const TERMINAL_ID: &str = "terminal";

#[derive(Clone, Copy)]
pub enum TabKind {
    Plot,
    Settings,
    Terminal,
}

pub struct Tab {
    pub kind: TabKind,
}

impl Tab {
    pub fn new(kind: TabKind) -> Self { Self { kind } }

    pub fn title(&self) -> &'static str {
        match self.kind {
            TabKind::Plot     => "Plot",
            TabKind::Settings => "Settings",
            TabKind::Terminal => "Terminal",
        }
    }

    pub fn citizen_id(&self) -> CitizenId {
        CitizenId::new(match self.kind {
            TabKind::Plot     => PLOT_ID,
            TabKind::Settings => SETTINGS_ID,
            TabKind::Terminal => TERMINAL_ID,
        })
    }
}

/// Bridge between `egui_dock` and the citizen layer.
///
/// `ui()` routes to each panel's render method. `on_tab_button` forwards
/// click events into `dispatcher.activate(...)` — the standard skeleton.
/// This app doesn't drive behavior off activation, but registering the
/// click is the canonical hook so the dispatcher's queue stays accurate.
pub struct TabViewer<'a> {
    pub state: &'a SharedState,
    pub dispatcher: &'a mut Dispatcher,
    pub plot: &'a mut PlotPanel,
    pub settings: &'a mut SettingsPanel,
    pub terminal: &'a mut TerminalPanel,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab.kind {
            TabKind::Plot     => self.plot.show(ui, self.state),
            TabKind::Settings => self.settings.show(ui, self.state, self.dispatcher),
            TabKind::Terminal => self.terminal.show(ui, self.state),
        }
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.clicked() {
            self.dispatcher.activate(&tab.citizen_id());
        }
    }
}
