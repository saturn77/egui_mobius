use tokio::sync::mpsc;
use tokio::task;
use std::sync::Arc;
use tokio::sync::Mutex;
use eframe::egui;
use tokio::time::Duration;

#[derive(Debug)]
pub enum SignalMessage {
    Command(String),
    Result(String),
}

#[derive(Clone)]
pub struct Signal {
    sender: mpsc::Sender<SignalMessage>,
    receiver: Arc<Mutex<mpsc::Receiver<SignalMessage>>>,
}

/// Create a Signal struct for communication between the frontend and backend
/// in an effort to make the system more responsive, and in particular, to allow
/// the GUI windowing close function to work properly. This formulation seems to
/// work well with the tokio runtime, and the tokio::spawn() function, allowing
/// the window to close without hanging. More experimentation will be needed. 
/// 21 Feb 2025, James B. 

impl Signal {
    // Create a new Signal with the sender and receiver
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let receiver = Arc::new(Mutex::new(receiver));

        Signal { sender, receiver }
    }

    // Send a command to the backend
    pub async fn send_command(&self, command: String) {
        let _ = self.sender.send(SignalMessage::Command(command)).await;
    }

    // Send a SignalMessage to the backend
    pub async fn send_message(&self, message: SignalMessage) {
        let _ = self.sender.send(message).await;
    }

    // Receive a result from the backend
    pub async fn receive_result(&self) -> Option<SignalMessage> {
        let mut receiver = self.receiver.lock().await;
        receiver.recv().await
    }
}

pub struct MyApp {
    signal: Signal,
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_ui(ctx);
    }
}

impl MyApp {
    fn new(signal: Signal) -> Self {
        Self { signal }
    }

    fn handle_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Send Command to Backend").clicked() {
                let signal_clone = self.signal.clone();
                tokio::spawn(async move {
                    signal_clone.send_command("Hello Backend!".to_string()).await;
                });
            }

            // Receive a result (blocking)
            tokio::spawn({
                let signal_clone = self.signal.clone();
                async move {
                    if let Some(result) = signal_clone.receive_result().await {
                        match result {
                            SignalMessage::Result(res) => {
                                eprintln!("Backend responded: {}", res);
                            }
                            _ => {}
                        }
                    }
                }
            });
        });
    }
}

pub fn backend(signal: Signal) {
    let signal_clone = signal.clone();

    task::spawn(async move {
        while let Some(message) = signal.receive_result().await {
            match message {
                SignalMessage::Command(cmd) => {
                    // Simulate backend work
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    let result = format!("Processed: {}", cmd);
                    signal_clone.send_message(SignalMessage::Result(result)).await;
                }
                SignalMessage::Result(res) => {
                    // Handle result message if needed
                    eprintln!("Received result: {}", res);
                }
            }
        }
    });
}

#[tokio::main]
async fn main() {
    // Create Signal instance for communication
    let signal = Signal::new();
    
    // Start the backend processing in a new task
    backend(signal.clone());

    // Set up the eframe app
    let app = MyApp::new(signal);
    if let Err(e) = eframe::run_native("Signal App", eframe::NativeOptions::default(), Box::new(|_cc| Ok(Box::new(app) as Box<dyn eframe::App>))) {
        eprintln!("Failed to run eframe: {}", e);
    }
}
