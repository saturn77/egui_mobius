//! filter_plotter — a citizen-pattern tutorial app.
//!
//! Three panels (Plot / Settings / Terminal) wired into `egui_dock` via a
//! `TabViewer`, with `egui_citizen::Registry` as the message hub
//! between the settings panel and an in-process IIR filter backend.
//!
//! Click Generate in the Settings panel → AppMessage::Generate flows
//! through the settings outbox → drain loop calls the backend → traces
//! land in SharedState → plot panel renders them on the next frame.

mod backend;
mod settings_actions;
mod messages;
mod citizens;
mod state;
mod tabs;
mod theme;

mod platform;

use eframe::egui;
use egui_citizen::Registry;
use egui_dock::{DockArea, DockState, NodeIndex};

use backend::iir::InProcessIir;
use citizens::{
    editor::EditorPanel, logger::LoggerPanel, plot::PlotPanel, settings::SettingsPanel,
};
use state::SharedState;
use tabs::{Tab, TabKind, TabViewer};

struct App {
    registry: Registry,
    dock_state: DockState<Tab>,
    state: SharedState,
    plot: PlotPanel,
    settings: SettingsPanel,
    logger: LoggerPanel,
    editor: EditorPanel,
    backend: InProcessIir,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_visuals(&cc.egui_ctx);

        // Register every citizen up front; panels hold the CitizenState
        // handles the registry gives back (never CitizenState::new()).
        let mut registry = Registry::new();
        let plot_state     = registry.add().with_name(tabs::PLOT_ID);
        let settings_state = registry.add().with_name(tabs::SETTINGS_ID);
        let logger_state   = registry.add().with_name(tabs::LOGGER_ID);
        let editor_state   = registry.add().with_name(tabs::EDITOR_ID);
        registry.activate(tabs::PLOT_ID);

        // Dock layout:
        //   ┌──────────────────┬─────────────┐
        //   │   Plot / Editor  │  Settings   │
        //   │   (tab group)    ├─────────────┤
        //   │                  │   Logger    │
        //   └──────────────────┴─────────────┘
        // Editor is grouped with Plot as a sibling tab so users can
        // toggle between viewing the filter result and editing
        // accompanying parameters / scripts.
        let mut dock_state = DockState::new(vec![
            Tab::new(TabKind::Plot),
            Tab::new(TabKind::Editor),
        ]);
        let [_, right] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.65,
            vec![Tab::new(TabKind::Settings)],
        );
        let [_, _bottom] =
            dock_state
                .main_surface_mut()
                .split_below(right, 0.55, vec![Tab::new(TabKind::Logger)]);

        let state = SharedState::new();
        citizens::append_log(&state, "filter_plotter started".into());

        Self {
            registry,
            dock_state,
            state,
            plot: PlotPanel::new(plot_state),
            settings: SettingsPanel::new(settings_state),
            logger: LoggerPanel::new(logger_state),
            editor: EditorPanel::new(editor_state),
            backend: InProcessIir::new(),
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // React to lens's "System Info" button — it sets a memory flag that
        // consuming apps must drain. Native: gather system details and log
        // them; wasm: log a one-line note since the underlying probes
        // (sysinfo, local-ip-address) don't work in browser.
        let show_system_info = ui.ctx().memory(|mem| {
            mem.data
                .get_temp::<bool>(egui::Id::new("show_system_info"))
                .unwrap_or(false)
        });
        if show_system_info {
            ui.ctx().memory_mut(|mem| {
                mem.data
                    .remove::<bool>(egui::Id::new("show_system_info"));
            });

            let mut details = platform::details::Details::new();
            let text = details.format_os();
            citizens::append_log(&self.state, text);
        }

        DockArea::new(&mut self.dock_state).show_inside(
            ui,
            &mut TabViewer {
                state: &self.state,
                registry: &mut self.registry,
                plot: &mut self.plot,
                settings: &mut self.settings,
                logger: &mut self.logger,
                editor: &mut self.editor,
            },
        );

        // Drain pass — once per frame, after the dock has rendered and
        // any on_tab_button or in-panel events have queued.
        citizens::drain_citizen(&mut self.registry, &self.state);

        let outbox = std::mem::take(&mut self.settings.outbox);
        for msg in outbox {
            settings_actions::handle(msg, &self.state, &mut self.backend);
        }
    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    
    eframe::run_native(
        "filter_plotter",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1200.0, 750.0])
                .with_min_inner_size([800.0, 500.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
