use std::sync::{Arc, Mutex};
use std::fmt::Display;
use std::sync::mpsc::Receiver;
use std::thread;


#[derive(Debug, Clone)]
pub struct Slot<T> {
    pub receiver: Arc<Mutex<Receiver<T>>>,
    pub sequence: usize,
}


impl<T> Slot<T>
where
    T: Send + 'static + Display + Clone,
{
    pub fn new(receiver: Receiver<T>) -> Self {
        Slot {
            receiver: Arc::new(Mutex::new(receiver)),
            sequence: 0,
        }
    }

    pub fn start<F>(&self, mut handler: F)
    where
        F: FnMut(T) + Send + 'static,
    {
        let receiver = Arc::clone(&self.receiver);
        thread::spawn(move || {
            let receiver = receiver.lock().unwrap();
            for command in receiver.iter() {
                handler(command);
            }
        });
    }
}