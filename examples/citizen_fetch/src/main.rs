//! Citizen Fetch — demonstrates backend threading with egui_citizen.
//!
//! - Fetch panel: enter a URL or use auto-fetch for random images
//! - Image panel: displays fetched images from picsum.photos
//! - Response panel: shows raw text responses
//! - Logger panel: shows citizen messages and fetch activity
//!
//! The backend thread performs HTTP requests while the UI stays responsive.
//! Auto-fetch cycles every N seconds, pulling random images.
//!
//! Run: cargo run -p citizen_fetch

use crossbeam_channel::{Receiver, Sender, unbounded};
use eframe::egui;
use egui::Color32;
use egui_citizen::message::CitizenId;
use egui_citizen::{CitizenMessage, Dispatcher};
use egui_dock::{DockArea, DockState, NodeIndex};
use std::time::{Duration, Instant};

// ── Colors (Tokyo Night subset) ─────────────────────────────────────────

const BG: Color32 = Color32::from_rgb(0x24, 0x28, 0x3b);
const FG: Color32 = Color32::from_rgb(0xc0, 0xca, 0xf5);
const CYAN: Color32 = Color32::from_rgb(0x7d, 0xcf, 0xff);
const GREEN: Color32 = Color32::from_rgb(0x9e, 0xce, 0x6a);
const RED: Color32 = Color32::from_rgb(0xf7, 0x76, 0x8e);
const COMMENT: Color32 = Color32::from_rgb(0x56, 0x5f, 0x89);
const ORANGE: Color32 = Color32::from_rgb(0xff, 0x9e, 0x64);

// ── Backend messages ────────────────────────────────────────────────────

enum FetchRequest {
    GetText(String),
    GetImage { url: String, id: u64 },
}

enum FetchResponse {
    Text {
        url: String,
        body: String,
        status: u16,
    },
    Image {
        id: u64,
        data: Vec<u8>,
        width: u32,
        height: u32,
    },
    Error {
        url: String,
        error: String,
    },
}

// ── Tabs ────────────────────────────────────────────────────────────────

#[derive(Clone)]
enum TabKind {
    Fetch,
    Image,
    Response,
    Logger,
}

#[derive(Clone)]
struct Tab {
    kind: TabKind,
}

impl Tab {
    fn title(&self) -> &str {
        match self.kind {
            TabKind::Fetch => "Fetch",
            TabKind::Image => "Image",
            TabKind::Response => "Response",
            TabKind::Logger => "Logger",
        }
    }

    fn citizen_id(&self) -> Option<CitizenId> {
        match self.kind {
            TabKind::Fetch => Some(CitizenId::new("fetch")),
            TabKind::Image => Some(CitizenId::new("image")),
            TabKind::Response => Some(CitizenId::new("response")),
            TabKind::Logger => None,
        }
    }
}

// ── Tab viewer ──────────────────────────────────────────────────────────

struct TabViewer<'a> {
    dispatcher: &'a mut Dispatcher,
    url: &'a mut String,
    request_tx: &'a Sender<FetchRequest>,
    response_body: &'a str,
    response_status: &'a str,
    is_fetching: &'a mut bool,
    auto_fetch: &'a mut bool,
    auto_interval_secs: &'a mut f32,
    image_texture: &'a Option<egui::TextureHandle>,
    image_info: &'a str,
    fetch_count: u64,
    log: &'a mut Vec<String>,
}

impl egui_dock::TabViewer for TabViewer<'_> {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn on_tab_button(&mut self, tab: &mut Tab, response: &egui::Response) {
        if response.clicked()
            && let Some(id) = tab.citizen_id()
        {
            self.dispatcher.activate(&id);
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Tab) {
        match tab.kind {
            TabKind::Fetch => self.render_fetch(ui),
            TabKind::Image => self.render_image(ui),
            TabKind::Response => self.render_response(ui),
            TabKind::Logger => self.render_logger(ui),
        }
    }
}

impl TabViewer<'_> {
    fn render_fetch(&mut self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(BG)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.heading(egui::RichText::new("HTTP Fetch").color(CYAN));
                ui.add_space(8.0);

                // Manual URL fetch
                ui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.text_edit_singleline(self.url);
                });

                ui.add_space(4.0);

                let button_text = if *self.is_fetching {
                    "Fetching..."
                } else {
                    "Fetch"
                };
                if ui
                    .add_enabled(!*self.is_fetching, egui::Button::new(button_text))
                    .clicked()
                {
                    let url = self.url.clone();
                    self.log.push(format!("[FETCH] Requesting: {}", url));
                    let _ = self.request_tx.send(FetchRequest::GetText(url));
                    *self.is_fetching = true;
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                // Auto-fetch random images
                ui.heading(egui::RichText::new("Auto Fetch Images").color(ORANGE));
                ui.add_space(4.0);

                ui.checkbox(self.auto_fetch, "Enable auto-fetch");

                ui.horizontal(|ui| {
                    ui.label("Interval:");
                    ui.add(
                        egui::Slider::new(self.auto_interval_secs, 2.0..=30.0)
                            .suffix("s")
                            .step_by(1.0),
                    );
                });

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(format!("Fetches: {}", self.fetch_count))
                        .color(GREEN)
                        .monospace(),
                );

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(
                        "Pulls random images from picsum.photos.\n\
                     Each fetch runs on a background thread.",
                    )
                    .color(COMMENT),
                );
            });
    }

    fn render_image(&self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(BG)
            .inner_margin(8.0)
            .show(ui, |ui| {
                if !self.image_info.is_empty() {
                    ui.label(egui::RichText::new(self.image_info).color(CYAN).monospace());
                    ui.separator();
                }

                if let Some(texture) = self.image_texture {
                    let available = ui.available_size();
                    let tex_size = texture.size_vec2();
                    let scale = (available.x / tex_size.x)
                        .min(available.y / tex_size.y)
                        .min(1.0);
                    let display_size = tex_size * scale;
                    ui.centered_and_justified(|ui| {
                        ui.image((texture.id(), display_size));
                    });
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(
                            egui::RichText::new(
                                "No image loaded yet.\nEnable auto-fetch or fetch a URL.",
                            )
                            .color(COMMENT),
                        );
                    });
                }
            });
    }

    fn render_response(&self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(BG)
            .inner_margin(12.0)
            .show(ui, |ui| {
                ui.heading(egui::RichText::new("Response").color(GREEN));
                ui.add_space(4.0);

                if !self.response_status.is_empty() {
                    ui.label(
                        egui::RichText::new(self.response_status)
                            .color(CYAN)
                            .monospace(),
                    );
                    ui.separator();
                }

                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.response_body.is_empty() {
                        ui.label(egui::RichText::new("No text response yet.").color(COMMENT));
                    } else {
                        ui.label(
                            egui::RichText::new(self.response_body)
                                .color(FG)
                                .monospace(),
                        );
                    }
                });
            });
    }

    fn render_logger(&self, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(BG)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.heading(egui::RichText::new("Messages").color(CYAN));
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in self.log.iter() {
                            let color = if line.contains("[CITIZEN]") {
                                GREEN
                            } else if line.contains("[ERROR]") {
                                RED
                            } else if line.contains("[IMAGE]") {
                                ORANGE
                            } else {
                                COMMENT
                            };
                            ui.label(egui::RichText::new(line).color(color).monospace());
                        }
                    });
            });
    }
}

// ── App ─────────────────────────────────────────────────────────────────

struct FetchApp {
    dock_state: DockState<Tab>,
    dispatcher: Dispatcher,
    url: String,
    response_body: String,
    response_status: String,
    is_fetching: bool,
    auto_fetch: bool,
    auto_interval_secs: f32,
    last_auto_fetch: Instant,
    fetch_count: u64,
    image_texture: Option<egui::TextureHandle>,
    image_info: String,
    log: Vec<String>,

    request_tx: Sender<FetchRequest>,
    response_rx: Receiver<FetchResponse>,
}

impl FetchApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut dispatcher = Dispatcher::new();
        dispatcher.register(CitizenId::new("fetch"));
        dispatcher.register(CitizenId::new("image"));
        dispatcher.register(CitizenId::new("response"));
        dispatcher.activate(&CitizenId::new("fetch"));
        let _ = dispatcher.drain_messages();

        // Layout:
        // ┌──────────┬───────────┐
        // │  Fetch   │  Image    │
        // ├──────────┼───────────┤
        // │  Logger  │ Response  │
        // └──────────┴───────────┘
        let mut dock_state = DockState::new(vec![Tab {
            kind: TabKind::Image,
        }]);
        let [left, right] = dock_state.main_surface_mut().split_left(
            NodeIndex::root(),
            0.30,
            vec![Tab {
                kind: TabKind::Fetch,
            }],
        );
        dock_state.main_surface_mut().split_below(
            left,
            0.65,
            vec![Tab {
                kind: TabKind::Logger,
            }],
        );
        dock_state.main_surface_mut().split_below(
            right,
            0.65,
            vec![Tab {
                kind: TabKind::Response,
            }],
        );

        let (request_tx, request_rx) = unbounded::<FetchRequest>();
        let (response_tx, response_rx) = unbounded::<FetchResponse>();

        // Backend thread
        std::thread::spawn(move || {
            for req in request_rx {
                match req {
                    FetchRequest::GetText(url) => match ureq::get(&url).call() {
                        Ok(resp) => {
                            let status = resp.status();
                            let body = resp
                                .into_string()
                                .unwrap_or_else(|e| format!("(read error: {})", e));
                            let body = if body.len() > 4000 {
                                format!(
                                    "{}...\n\n[truncated, {} bytes total]",
                                    &body[..4000],
                                    body.len()
                                )
                            } else {
                                body
                            };
                            let _ = response_tx.send(FetchResponse::Text { url, body, status });
                        }
                        Err(e) => {
                            let _ = response_tx.send(FetchResponse::Error {
                                url,
                                error: e.to_string(),
                            });
                        }
                    },
                    FetchRequest::GetImage { url, id } => {
                        match ureq::get(&url).call() {
                            Ok(resp) => {
                                let mut bytes = Vec::new();
                                if let Err(e) = resp.into_reader().read_to_end(&mut bytes) {
                                    let _ = response_tx.send(FetchResponse::Error {
                                        url,
                                        error: format!("Read error: {}", e),
                                    });
                                    continue;
                                }
                                // Decode image to get dimensions
                                match image::load_from_memory(&bytes) {
                                    Ok(img) => {
                                        let rgba = img.to_rgba8();
                                        let (w, h) = rgba.dimensions();
                                        let _ = response_tx.send(FetchResponse::Image {
                                            id,
                                            data: rgba.into_raw(),
                                            width: w,
                                            height: h,
                                        });
                                    }
                                    Err(e) => {
                                        let _ = response_tx.send(FetchResponse::Error {
                                            url,
                                            error: format!("Image decode error: {}", e),
                                        });
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = response_tx.send(FetchResponse::Error {
                                    url,
                                    error: e.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        });

        Self {
            dock_state,
            dispatcher,
            url: "https://httpbin.org/get".to_string(),
            response_body: String::new(),
            response_status: String::new(),
            is_fetching: false,
            auto_fetch: false,
            auto_interval_secs: 5.0,
            last_auto_fetch: Instant::now(),
            fetch_count: 0,
            image_texture: None,
            image_info: String::new(),
            log: vec!["[INFO] Citizen Fetch example started".to_string()],
            request_tx,
            response_rx,
        }
    }
}

impl eframe::App for FetchApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Process responses from backend thread
        while let Ok(response) = self.response_rx.try_recv() {
            self.is_fetching = false;
            match response {
                FetchResponse::Text { url, body, status } => {
                    self.log.push(format!(
                        "[FETCH] {} → {} ({} bytes)",
                        url,
                        status,
                        body.len()
                    ));
                    self.response_status = format!("HTTP {} — {}", status, url);
                    self.response_body = body;
                }
                FetchResponse::Image {
                    id,
                    data,
                    width,
                    height,
                } => {
                    self.log
                        .push(format!("[IMAGE] #{} received — {}x{}", id, width, height));
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [width as usize, height as usize],
                        &data,
                    );
                    self.image_texture = Some(ui.load_texture(
                        format!("fetched_image_{}", id),
                        color_image,
                        egui::TextureOptions::LINEAR,
                    ));
                    self.image_info = format!("Image #{} — {}x{}", id, width, height);
                }
                FetchResponse::Error { url, error } => {
                    self.log.push(format!("[ERROR] {} — {}", url, error));
                    self.response_status = format!("Error — {}", url);
                    self.response_body = error;
                }
            }
        }

        // Auto-fetch timer
        if self.auto_fetch {
            let elapsed = self.last_auto_fetch.elapsed();
            if elapsed >= Duration::from_secs_f32(self.auto_interval_secs) && !self.is_fetching {
                self.fetch_count += 1;
                let url = format!("https://picsum.photos/600/400?random={}", self.fetch_count);
                self.log
                    .push(format!("[IMAGE] Auto-fetch #{}: {}", self.fetch_count, url));
                let _ = self.request_tx.send(FetchRequest::GetImage {
                    url,
                    id: self.fetch_count,
                });
                self.is_fetching = true;
                self.last_auto_fetch = Instant::now();
            }
        }

        // Render dock
        let mut dock_state = self.dock_state.clone();
        let mut dispatcher = std::mem::take(&mut self.dispatcher);
        {
            let mut viewer = TabViewer {
                dispatcher: &mut dispatcher,
                url: &mut self.url,
                request_tx: &self.request_tx,
                response_body: &self.response_body,
                response_status: &self.response_status,
                is_fetching: &mut self.is_fetching,
                auto_fetch: &mut self.auto_fetch,
                auto_interval_secs: &mut self.auto_interval_secs,
                image_texture: &self.image_texture,
                image_info: &self.image_info,
                fetch_count: self.fetch_count,
                log: &mut self.log,
            };
            DockArea::new(&mut dock_state).show_inside(ui, &mut viewer);
        }

        // Drain citizen messages
        for msg in dispatcher.drain_messages() {
            match &msg {
                CitizenMessage::Activated { id } => {
                    self.log.push(format!("[CITIZEN] Activated: {}", id));
                }
                CitizenMessage::Deactivated { id } => {
                    self.log.push(format!("[CITIZEN] Deactivated: {}", id));
                }
                _ => {}
            }
        }

        self.dispatcher = dispatcher;
        self.dock_state = dock_state;

        // Keep repainting during fetch or auto-fetch
        if self.is_fetching || self.auto_fetch {
            ui.ctx().request_repaint();
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Citizen Fetch",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([900.0, 600.0])
                .with_min_inner_size([600.0, 400.0]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(FetchApp::new(cc)))),
    )
}
