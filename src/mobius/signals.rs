#[allow(dead_code)]
use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};

pub trait CommandTrait: Send + 'static {}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum CommandResult {
    Success(String),
    Failure(String),
}
#[allow(dead_code)]
#[derive(Clone)]
pub struct Signal<T> {
    pub sender: mpsc::Sender<T>,
    pub receiver: Arc<Mutex<Option<T>>>,
    pub result_receiver: Arc<Mutex<Option<CommandResult>>>,
}

#[allow(dead_code)]
impl<T> Signal<T> {
    pub fn get_result(&self) -> Option<CommandResult> {
        self.result_receiver.lock().unwrap().clone()
    }
}

#[allow(dead_code)]
impl<C: CommandTrait> Signal<C> {
    pub fn new() -> (Self, mpsc::Receiver<C>) {
        let (command_sender, command_receiver) = mpsc::channel::<C>(32);
        (Self { sender: command_sender, receiver: Arc::new(Mutex::new(None)), result_receiver: Arc::new(Mutex::new(None)) }, command_receiver)
    }

    pub async fn send_command(&self, command: C) -> Result<(), mpsc::error::SendError<C>> {
        self.sender.send(command).await
    }
}