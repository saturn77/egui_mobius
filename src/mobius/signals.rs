#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use std::sync::mpsc;

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

impl<T: Send + 'static > Signal<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        Signal { sender, receiver }
    }

    pub fn send_command(&self, message: T) {
        self.sender.send(WireType::Command(message)).unwrap();
    }

    pub fn send_result(&self, message: T) {
        self.sender.send(WireType::Result(message)).unwrap();
    }

    // Non-blocking receive function: Returns None if no message is available
    pub fn try_receive(&self) -> Option<WireType<T>> {
        let receiver = self.receiver.lock().unwrap();
        receiver.try_recv().ok()
    }
}