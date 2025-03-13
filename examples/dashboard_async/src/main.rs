// examples/dashboard/main.rs
use egui::Context;
use egui_mobius::factory;
use egui_mobius::signals::*;
use egui_mobius::slot::*;
use egui_mobius::types::*;
use std::sync::Arc;
use tokio::runtime::Runtime;

#[derive(Clone)]
pub enum Event {
    FetchBitcoin,
    FetchKaspa,
    FetchSolana,
    FetchStellar,
    FetchSui,
}

#[derive(Clone)]
pub enum Processed {
    BitcoinPrice(f64),
    KaspaPrice(f64),
    SolanaPrice(f64),
    StellarPrice(f64),
    SuiPrice(f64),
}
/// AppState
///
/// AppState is a struct that holds the state of the application
/// that is shared between the UI and the background dispatcher.
/// The background dispatcher is connected to the AppState via a Signal
/// and a Slot, and the UI is connected to the AppState via a Value.
///
#[derive(Debug, Clone)]
pub struct AppState {
    pub bitcoin_price: Option<f64>,
    pub kaspa_price: Option<f64>,
    pub solana_price: Option<f64>,
    pub stellar_price: Option<f64>,
    pub sui_price: Option<f64>,
    pub loading_coin: Option<String>,
    pub spinner_angle: f32,
    pub error_message: Option<String>,
    pub price_log: Vec<String>,
}

impl AppState {
    pub fn new(_event_signal: Signal<Event>) -> Self {
        Self {
            bitcoin_price: None,
            kaspa_price: None,
            solana_price: None,
            stellar_price: None,
            sui_price: None,
            loading_coin: None,
            spinner_angle: 0.0,
            error_message: None,
            price_log: Vec::new(),
        }
    }
    pub fn log_price(&mut self, symbol: &str, price: f64) {
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        self.price_log
            .push(format!("[{}] {}: ${:.6}", timestamp, symbol, price));
        if self.price_log.len() > 1000 {
            self.price_log.drain(0..self.price_log.len() - 1000);
        }
    }

    fn handle_processed(&mut self, processed: Processed) {
        match processed {
            Processed::BitcoinPrice(p) => {
                self.loading_coin = None;
                if p > 0.0 {
                    self.bitcoin_price = Some(p);
                    self.error_message = None;
                    self.log_price("BTC", p);
                } else {
                    self.bitcoin_price = None;
                    self.error_message =
                        Some("Failed to retrieve a valid Bitcoin price.".to_string());
                }
            }
            Processed::KaspaPrice(p) => {
                self.loading_coin = None;
                if p > 0.0 {
                    self.kaspa_price = Some(p);
                    self.error_message = None;
                    self.log_price("KAS", p);
                } else {
                    self.kaspa_price = None;
                    self.error_message =
                        Some("Failed to retrieve a valid Kaspa price.".to_string());
                }
            }

            Processed::SolanaPrice(p) => {
                self.loading_coin = None;
                if p > 0.0 {
                    self.solana_price = Some(p);
                    self.error_message = None;
                    self.log_price("SOL", p);
                } else {
                    self.solana_price = None;
                    self.error_message =
                        Some("Failed to retrieve a valid Solana price.".to_string());
                }
            }
            Processed::StellarPrice(p) => {
                self.loading_coin = None;
                if p > 0.0 {
                    self.stellar_price = Some(p);
                    self.error_message = None;
                    self.log_price("XLM", p);
                } else {
                    self.stellar_price = None;
                    self.error_message =
                        Some("Failed to retrieve a valid Stellar price.".to_string());
                }
            }
            Processed::SuiPrice(p) => {
                self.loading_coin = None;
                if p > 0.0 {
                    self.sui_price = Some(p);
                    self.error_message = None;
                    self.log_price("SUI", p);
                } else {
                    self.sui_price = None;
                    self.error_message = Some("Failed to retrieve a valid SUI price.".to_string());
                }
            }
        }
    }
}

/// UiMainWindow
/// 
/// The UiMainWindow struct is the main UI window for the application.
/// It is responsible for rendering the UI and handling user input.
/// The UiMainWindow struct holds a Value<AppState> that is shared with the
/// background dispatcher via a Signal<Event> and a Slot<Processed>.
///
/// The UiMainWindow gets it's name as inspired by the UiMainWindow in Qt, which is the main window
/// of a Qt application. The UiMainWindow is the main window of the application, and is responsible
/// for rendering the UI and handling user input.
///
#[derive(Clone)]
pub struct UiMainWindow {
    pub state: Value<AppState>,
    pub event_signal: Signal<Event>,
}
/// Register the Slot for the UiMainWindow in the new method
/// and then you can call the handle_processed method on the UiMainWindow instance
impl UiMainWindow {
    pub fn new(event_signal: Signal<Event>, mut response_slot: Slot<Processed>) -> Self {
        let state = Value::new(AppState::new(event_signal.clone()));

        // Initialize the response listener exactly once
        let state_clone = state.clone();
        response_slot.start(move |response| {
            state_clone.lock().unwrap().handle_processed(response);
        });

        Self {
            state,
            event_signal,
        }
    }
}

impl eframe::App for UiMainWindow {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Cryptocurrency Prices");
                ui.label("Powered by egui_mobius");
            });

            ui.add_space(10.0);

            if ui.button("Fetch Bitcoin Price").clicked() {
                let mut state = self.state.lock().unwrap();
                state.loading_coin = Some("BTC".to_string());
                let _ = self.event_signal.send(Event::FetchBitcoin);
            }
            ui.add_space(10.0);

            if ui.button("Fetch Kaspa Price").clicked() {
                let mut state = self.state.lock().unwrap();
                state.loading_coin = Some("KAS".to_string());
                let _ = self.event_signal.send(Event::FetchKaspa);
            }

            ui.add_space(10.0);

            if ui.button("Fetch Solana Price").clicked() {
                let mut state = self.state.lock().unwrap();
                state.loading_coin = Some("SOL".to_string());
                let _ = self.event_signal.send(Event::FetchSolana);
            }

            ui.add_space(10.0);

            if ui.button("Fetch Stellar Price").clicked() {
                let mut state = self.state.lock().unwrap();
                state.loading_coin = Some("XLM".to_string());
                let _ = self.event_signal.send(Event::FetchStellar);
            }

            ui.add_space(10.0);

            if ui.button("Fetch SUI Price").clicked() {
                let mut state = self.state.lock().unwrap();
                state.loading_coin = Some("SUI".to_string());
                let _ = self.event_signal.send(Event::FetchSui);
            }

            ui.separator();

            ui.label("Price Log:");
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    let price_log = &self.state.lock().unwrap().price_log;
                    for entry in price_log.iter() {
                        let color = if entry.contains("BTC") {
                            egui::Color32::YELLOW
                        } else if entry.contains("KAS") {
                            egui::Color32::GREEN
                        } else if entry.contains("SOL") {
                            egui::Color32::LIGHT_BLUE
                        } else if entry.contains("XLM") {
                            egui::Color32::WHITE
                        } else if entry.contains("SUI") {
                            egui::Color32::LIGHT_RED
                        } else {
                            egui::Color32::GRAY
                        };
                        ui.colored_label(color, entry);
                    }
                });
            let state = self.state.lock().unwrap();

            if let Some(ref loading) = state.loading_coin {
                ui.horizontal(|ui| {
                    ui.label(format!("Loading {} price...", loading));
                });
                ui.horizontal(|ui| {
                    ui.label("Loading price...");
                });
            }
            // must match all the way to the end of the match statement
            else if let Some(price) = state.sui_price {
                ui.label(format!("SUI Price: ${:.2}", price));
            } else if let Some(price) = state.stellar_price {
                ui.label(format!("Stellar Price: ${:.2}", price));
            } else if let Some(price) = state.solana_price {
                ui.label(format!("Solana Price: ${:.2}", price));
            } else if let Some(price) = state.kaspa_price {
                ui.label(format!("Kaspa Price: ${:.2}", price));
            } else if let Some(price) = state.bitcoin_price {
                ui.label(format!("Bitcoin Price: ${:.2}", price));
            } else if let Some(ref err) = state.error_message {
                ui.colored_label(egui::Color32::RED, err);
            }

            // TODO : add a timeout for the loading spinner, i.e. if the request takes too long
            // then show an error message
        });
    }
}

fn main() {
    let (signal_to_dispatcher, slot_from_ui) = factory::create_signal_slot::<Event>(1);
    let (signal_to_ui, slot_from_dispatcher) = factory::create_signal_slot::<Processed>(1);

    let app = UiMainWindow::new(signal_to_dispatcher, slot_from_dispatcher);

    let mut dispatcher = Dispatcher::new();
    dispatcher.run(slot_from_ui, signal_to_ui.clone());
    //dispatcher.

    //start_dispatcher(slot_from_ui, signal_to_ui.clone());

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_titlebar_buttons_shown(true)
            .with_min_inner_size((750.0, 500.0))
            .with_resizable(true),
        ..Default::default()
    };

    if let Err(e) = eframe::run_native(
        "Dashboard with egui_mobius",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    ) {
        eprintln!("Failed to run eframe UiMainWindowlication: {:?}", e);
    }
}

/// Dispatcher
///
/// The Dispatcher struct is responsible for running the background tasks
/// that fetch the cryptocurrency prices. It listens for events on a Slot
/// and sends the processed results to a Signal.
///
/// The Events arriving via the slot are assigned (brokered) to the appropriate
/// fetch_price method, which is a Tokio async function that fetches the price
/// from the Kraken API. The results are then sent to the Signal.
///
/// The general flow :
/// Signals from Ui => Dispatcher (assign Event to a "task" method, which is a fetch_price method)
/// TokioRuntime => fetch_price method => Signal => Ui
///
/// It would be relativeley easy to extend the functionality of the Dispatcher
/// to include any number of other background tasks, such as fetching news articles,
/// or fetching data from other APIs.
///
/// The are currently no other methods other than run, however, it would be easy to add
/// a method to stop the dispatcher, or to add a method to add more tasks to the dispatcher.
/// A "Broker" method could be added to the Dispatcher to handle the assignment of tasks to
/// the appropriate fetch_price method (queueing, etc).
///
/// The Dispatcher could also be extended to include a method to handle the results of the
/// fetch_price methods, such as logging the results, or sending the results to a database.
///
struct Dispatcher {}

impl Dispatcher {
    pub fn new() -> Self {
        Self {}
    }

    fn run(&mut self, mut slot: Slot<Event>, signal: Signal<Processed>) {
        let runtime = Arc::new(Runtime::new().expect("Failed to build Tokio runtime"));

        slot.start(move |event| {
            let signal = signal.clone();
            let runtime = runtime.clone();
            match event {
                Event::FetchBitcoin => {
                    runtime.spawn(async move {
                        let price = fetch_price("BTCUSD").await;
                        if let Err(e) = signal.send(Processed::BitcoinPrice(price)) {
                            eprintln!("Failed to send Bitcoin price: {:?}", e);
                        }
                    });
                }
                Event::FetchKaspa => {
                    runtime.spawn(async move {
                        let price = fetch_price("KASUSD").await;
                        if let Err(e) = signal.send(Processed::KaspaPrice(price)) {
                            eprintln!("Failed to send Kaspa price: {:?}", e);
                        }
                    });
                }
                Event::FetchSolana => {
                    runtime.spawn(async move {
                        let price = fetch_price("SOLUSD").await;
                        if let Err(e) = signal.send(Processed::SolanaPrice(price)) {
                            eprintln!("Failed to send Solana price: {:?}", e);
                        }
                    });
                }
                Event::FetchStellar => {
                    runtime.spawn(async move {
                        let price = fetch_price("XLMUSD").await;
                        if let Err(e) = signal.send(Processed::StellarPrice(price)) {
                            eprintln!("Failed to send Stellar price: {:?}", e);
                        }
                    });
                }
                Event::FetchSui => {
                    runtime.spawn(async move {
                        let price = fetch_price("SUIUSD").await;
                        if let Err(e) = signal.send(Processed::SuiPrice(price)) {
                            eprintln!("Failed to send SUI price: {:?}", e);
                        }
                    });
                }
            }
        });
    }
}

#[derive(serde::Deserialize, Debug)]
struct BitcoinPrice {
    result: std::collections::HashMap<String, KrakenTicker>,
}

#[derive(serde::Deserialize, Debug)]
struct KrakenTicker {
    c: [String; 2],
}

async fn fetch_price(pair: &str) -> f64 {
    let url = format!("https://api.kraken.com/0/public/Ticker?pair={}", pair);

    match reqwest::get(&url).await {
        Ok(resp) => match resp.json::<BitcoinPrice>().await {
            Ok(json) => {
                if let Some(ticker) = json.result.values().next() {
                    if let Ok(price) = ticker.c[0].parse::<f64>() {
                        println!("{} price: ${:.6}", pair, price);
                        return price;
                    } else {
                        eprintln!("Failed to parse price string: {:?}", ticker.c[0]);
                    }
                } else {
                    eprintln!("No ticker data found in Kraken result");
                }
            }
            Err(e) => {
                eprintln!("Failed to parse Kraken JSON: {:?}", e);
            }
        },
        Err(e) => {
            eprintln!("HTTP request error: {:?}", e);
        }
    }

    -1.0
}
