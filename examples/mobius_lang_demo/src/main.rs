//! A `.mobius` file on screen — real citizens, as plugins, in a dock.
//!
//! The full pipeline, live: `app.mobius` is parsed, checked, lowered to the
//! IR netlist, and wired into real `Dynamic<T>` values and signal queues.
//! Every instance becomes an `egui_dock` tab registered with the
//! `egui_citizen` `Dispatcher`. Registry citizens (`PlotPanel`, `LensLogger`)
//! are built by **plugins** that wrap the real `egui_plot` and `egui_lens`
//! crates (see `plugins.rs`); the source-declared `ControlsPanel` is drawn
//! generically from its IR widget tree by `mobius_lang_host::render_source`.
//!
//! Repaints are event-driven — the app idles at zero CPU. Edit `app.mobius`
//! while it runs: it re-elaborates on save. A broken edit keeps the last good
//! app running with the error shown.

mod plugins;

use std::any::Any;
use std::path::PathBuf;
use std::time::SystemTime;

use eframe::egui;
use egui_citizen::{CitizenId, Dispatcher};
use egui_dock::{DockArea, DockState, NodeIndex};
use mobius_lang::ir::InstanceKind;
use mobius_lang::{Host, Registry, WiredApp, lower, parse, wire};
use mobius_lang_host::{CitizenView, EventResolver, Plugins, render_source};

use plugins::{LensLogger, PlotPanel};

/// The one compiled event type. `BenchCmd::Apply` in source resolves here.
#[derive(Debug)]
enum BenchCmd {
    Apply,
}

/// The host owns the event types; this turns an event path from source into
/// the boxed compiled event a signal carries.
fn resolve_event(name: &str) -> Option<Box<dyn Any + Send>> {
    match name {
        "BenchCmd::Apply" => Some(Box::new(BenchCmd::Apply)),
        _ => None,
    }
}

fn plugins() -> Plugins {
    Plugins::new()
        .register("PlotPanel", PlotPanel::build)
        .register("LensLogger", LensLogger::build)
}

fn source_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("app.mobius")
}

fn build(path: &PathBuf, plugins: &Plugins) -> Result<WiredApp, String> {
    let source = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    // The language validates instantiations against exactly the plugins the
    // host provides, plus the host's handler and event names.
    let mut registry = Registry::new().handler("bench_worker").event("BenchCmd");
    for name in plugins.names() {
        registry = registry.citizen(name);
    }
    let (file, _comments) = parse(&source).map_err(|e| e.to_string())?;
    let app = file.apps.first().ok_or("no `app` in source")?;
    let ir = lower(app, &registry).map_err(|diagnostics| {
        diagnostics
            .iter()
            .map(|d| d.to_string())
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    wire(&ir, &Host::new()).map_err(|e| e.to_string())
}

fn modified(path: &PathBuf) -> Option<SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

// ----------------------------------------------------------------------
// Dock: each instance is a citizen tab; @layout drives the splits
// ----------------------------------------------------------------------

struct Tab {
    instance: usize,
    title: String,
}

fn build_dock(wired: &WiredApp) -> (DockState<Tab>, Dispatcher) {
    let mut dispatcher = Dispatcher::new();
    for instance in &wired.instances {
        dispatcher.register(CitizenId::new(instance.name.clone()));
    }

    let tab = |instance: usize| Tab {
        instance,
        title: wired.instances[instance].name.clone(),
    };

    let center = wired
        .layout
        .iter()
        .find(|dock| !matches!(dock.region.as_str(), "left" | "right" | "above" | "below"))
        .map(|dock| dock.instance)
        .unwrap_or(0);
    let mut dock_state = DockState::new(vec![tab(center)]);
    dispatcher.activate(&CitizenId::new(wired.instances[center].name.clone()));

    // Horizontal strips (above/below) carve the full width first so the
    // logger spans the bottom; the surviving band takes the left/right
    // splits. Source `fraction` is the new node's share; egui_dock's
    // fraction is the share the old node keeps.
    let mut band = NodeIndex::root();
    for pass in [["above", "below"], ["left", "right"]] {
        for dock in &wired.layout {
            if !pass.contains(&dock.region.as_str()) {
                continue;
            }
            let share = dock.fraction.unwrap_or(0.25) as f32;
            let keep = (1.0 - share).clamp(0.05, 0.95);
            let tabs = vec![tab(dock.instance)];
            let surface = dock_state.main_surface_mut();
            let split = match dock.region.as_str() {
                "above" => surface.split_above(band, keep, tabs),
                "below" => surface.split_below(band, keep, tabs),
                "left" => surface.split_left(band, keep, tabs),
                "right" => surface.split_right(band, keep, tabs),
                _ => continue,
            };
            band = split[0];
        }
    }

    (dock_state, dispatcher)
}

struct TabViewer<'a> {
    wired: &'a WiredApp,
    views: &'a mut [Option<Box<dyn CitizenView>>],
    dispatcher: &'a mut Dispatcher,
    events: &'a EventResolver<'a>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        let active = self
            .dispatcher
            .get(&CitizenId::new(tab.title.clone()))
            .map(|state| state.active.get())
            .unwrap_or(false);
        if active {
            format!("● {}", tab.title).into()
        } else {
            tab.title.clone().into()
        }
    }

    fn on_tab_button(&mut self, tab: &mut Self::Tab, response: &egui::Response) {
        if response.clicked() {
            // One-hot lifecycle activation through the citizen dispatcher.
            self.dispatcher.activate(&CitizenId::new(tab.title.clone()));
            let _ = self.dispatcher.drain_messages();
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match &mut self.views[tab.instance] {
            // Plugin citizen: wraps a real crate, draws itself.
            Some(view) => view.ui(ui),
            // Source citizen: rendered generically from its IR widgets.
            None => {
                let instance = &self.wired.instances[tab.instance];
                render_source(ui, &instance.widgets, &instance.bindings, self.events);
            }
        }
    }
}

// ----------------------------------------------------------------------
// Application
// ----------------------------------------------------------------------

struct DemoApp {
    path: PathBuf,
    plugins: Plugins,
    wired: Option<WiredApp>,
    views: Vec<Option<Box<dyn CitizenView>>>,
    dock_state: Option<DockState<Tab>>,
    dispatcher: Dispatcher,
    error: Option<String>,
    last_modified: Option<SystemTime>,
    reloads: u32,
}

impl DemoApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let path = source_path();
        let last_modified = modified(&path);

        // File watcher: wake the UI when the source changes; the UI thread
        // does the reload. Repaints happen on input, results, and this.
        let ctx = cc.egui_ctx.clone();
        let watch_path = path.clone();
        let mut watched = last_modified;
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                let now = modified(&watch_path);
                if now != watched {
                    watched = now;
                    ctx.request_repaint();
                }
            }
        });

        let mut app = Self {
            path,
            plugins: plugins(),
            wired: None,
            views: Vec::new(),
            dock_state: None,
            dispatcher: Dispatcher::new(),
            error: None,
            last_modified,
            reloads: 0,
        };
        app.elaborate(false);
        app
    }

    /// Build (or rebuild) the wired app, plugin views, dock, and dispatcher.
    fn elaborate(&mut self, is_reload: bool) {
        match build(&self.path, &self.plugins) {
            Ok(wired) => {
                self.views = wired
                    .instances
                    .iter()
                    .map(|instance| match instance.kind {
                        InstanceKind::Registry => self.plugins.build(instance),
                        InstanceKind::Source => None,
                    })
                    .collect();
                let (dock_state, dispatcher) = build_dock(&wired);
                self.wired = Some(wired);
                self.dock_state = Some(dock_state);
                self.dispatcher = dispatcher;
                self.error = None;
                if is_reload {
                    self.reloads += 1;
                }
            }
            Err(error) => self.error = Some(error),
        }
    }

    fn watch(&mut self) {
        let now = modified(&self.path);
        if now != self.last_modified {
            self.last_modified = now;
            self.elaborate(true);
        }
    }

    /// The demo's `bench_worker` handler: drain commands, answer through
    /// `trace`. In a real host this runs on a backend thread; the wiring is
    /// identical (all handles are `Send`).
    fn drive_backend(&self) {
        let Some(wired) = &self.wired else { return };
        for handler in &wired.handlers {
            if handler.name != "bench_worker" {
                continue;
            }
            let applies: usize = handler
                .drains
                .iter()
                .map(|drain| drain.drain::<BenchCmd>().len())
                .sum();
            if applies == 0 {
                continue;
            }
            let (Some(amplitude), Some(enabled), Some(trace)) = (
                handler.bindings.read::<f32>("bench.amplitude"),
                handler.bindings.read::<bool>("bench.enabled"),
                handler.bindings.write::<Vec<f32>>("bench.trace"),
            ) else {
                continue;
            };
            if !enabled.get() {
                trace.set(Vec::new());
                continue;
            }
            let amp = amplitude.get();
            let samples: Vec<f32> = (0..512)
                .map(|i| amp * (i as f32 * std::f32::consts::TAU / 128.0).sin())
                .collect();
            trace.set(samples);
        }
    }
}

impl eframe::App for DemoApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.watch();
        self.drive_backend();

        egui::Panel::top("demo_status").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("source:");
                ui.monospace(self.path.display().to_string());
                ui.label(format!("· reloads: {}", self.reloads));
                if self.error.is_none() {
                    ui.colored_label(egui::Color32::LIGHT_GREEN, "live");
                }
            });
            if let Some(error) = &self.error {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    format!("edit kept last good app — {error}"),
                );
            }
        });

        let DemoApp {
            wired,
            views,
            dock_state,
            dispatcher,
            ..
        } = self;

        let (Some(wired), Some(dock_state)) = (wired.as_ref(), dock_state.as_mut()) else {
            ui.heading("no application");
            return;
        };

        let events: &EventResolver = &resolve_event;
        DockArea::new(dock_state).show_inside(
            ui,
            &mut TabViewer {
                wired,
                views,
                dispatcher,
                events,
            },
        );
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([700.0, 450.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "mobius_lang demo",
        options,
        Box::new(|cc| Ok(Box::new(DemoApp::new(cc)))),
    )
}
