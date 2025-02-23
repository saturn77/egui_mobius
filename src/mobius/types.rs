
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub type MobiusString = Arc<Mutex<String>>;
pub type MobiusSender<T> = mpsc::Sender<T>; 
pub type MobiusReceiver<T> = mpsc::Receiver<T>;

pub struct MobiusCommandTypes<T> {
    pub command_sender: MobiusSender<T>,
    pub result_receiver: MobiusReceiver<T>,
}