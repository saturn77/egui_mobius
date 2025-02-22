#![allow(dead_code)]

use std::sync::{Arc, Mutex};

use std::sync::mpsc;

// Backend (business logic) will be handled in a separate thread
#[derive(Debug)]
pub enum SignalMessage<T> {
    Command(T),
    Result(T),
}

#[derive(Clone, Debug)]
pub struct Signal<T> {
    sender: mpsc::Sender<SignalMessage<T>>,
    receiver: Arc<Mutex<mpsc::Receiver<SignalMessage<T>>>>,
}

impl<T> Signal<T> {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Signal {
            sender,
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    pub fn send_command(&self, message: T) {
        self.sender.send(SignalMessage::Command(message)).unwrap();
    }

    pub fn send_result(&self, message: T) {
        self.sender.send(SignalMessage::Result(message)).unwrap();
    }

    // Non-blocking receive function: Returns None if no message is available
    pub fn try_receive(&self) -> Option<SignalMessage<T>> {
        let receiver = self.receiver.lock().unwrap();
        receiver.try_recv().ok()
    }
}