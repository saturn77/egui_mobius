use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;

pub struct Slot<T> {
    pub receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T> Slot<T>
where
    T: Send + 'static,
{
    pub fn new(receiver: Receiver<T>) -> Self {
        Slot {
            receiver: Arc::new(Mutex::new(receiver)),
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