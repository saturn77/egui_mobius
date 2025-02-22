#![allow(dead_code)]

use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

// Backend (business logic) will be handled in a separate thread
#[derive(Debug, Clone)]
pub enum WireType<T> {
    Command(T),
    Result(T),
}

#[derive(Clone, Debug)]
pub struct Signal<T> {
    pub sender: mpsc::Sender<WireType<T>>,
    pub receiver: Arc<Mutex<mpsc::Receiver<WireType<T>>>>,
}

impl<T: Send + 'static> Signal<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(32);
        let receiver = Arc::new(Mutex::new(receiver));
        Signal { sender, receiver }
    }

    pub async fn send_command(&self, message: T) {
        self.sender.send(WireType::Command(message)).await.unwrap();
    }

    pub async fn send_result(&self, message: T) {
        self.sender.send(WireType::Result(message)).await.unwrap();
    }

    // Non-blocking receive function: Returns None if no message is available
    pub async fn try_receive(&self) -> Option<WireType<T>> {
        let mut receiver = self.receiver.lock().await;
        receiver.try_recv().ok()
    }
}