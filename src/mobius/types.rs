
use std::sync::{Arc, Mutex};

pub type MobiusCommandEnque<T> = crossbeam_channel::Sender<T>;
pub type MobiusCommandDeque<T> = crossbeam_channel::Receiver<T>;

pub type MobiusEventEnque<T> = tokio::sync::mpsc::Sender<T>;
pub type MobiusEventDeque<T> = tokio::sync::mpsc::Receiver<T>;


pub type MobiusString      = Arc<Mutex<String>>;
//pub type MobiusSender<T>   = mpsc::Sender<T>; 
//pub type MobiusReceiver<T> = mpsc::Receiver<T>;

