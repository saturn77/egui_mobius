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
    pub fn new(receiver: Receiver<T>, id_sequence : Option<usize>) -> Self {
        Slot {
            receiver: Arc::new(Mutex::new(receiver)),
            sequence: id_sequence.unwrap_or(0),
        }
    }

    pub fn start<F>(&self, mut handler: F)
    where
        F: FnMut(T) + Send + 'static + Clone + Display,
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

impl Display for Slot<String> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot: {}", self.sequence)
    }
}

impl Display for Slot<i32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot: {}", self.sequence)
    }
}

impl Display for Slot<f32> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot: {}", self.sequence)
    }
}