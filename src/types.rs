
use std::sync::{Arc, Mutex};

pub type MobiusEnque<T> = std::sync::mpsc::Sender<T>;
pub type MobiusDeque<T> = std::sync::mpsc::Receiver<T>;

pub type MobiusEventEnque<T> = tokio::sync::mpsc::Sender<T>;
pub type MobiusEventDeque<T> = tokio::sync::mpsc::Receiver<T>;

pub type MobiusString      = Arc<Mutex<String>>;
//pub type MobiusSender<T>   = mpsc::Sender<T>; 
//pub type MobiusReceiver<T> = mpsc::Receiver<T>;

