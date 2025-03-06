use std::sync::{Arc, Mutex};
use std::fmt::Display;
use std::sync::mpsc::Receiver;
use std::thread;


#[derive(Debug)]
pub struct Slot<T> {
    pub receiver: Arc<Mutex<Receiver<T>>>,
    pub sequence: usize,
}

impl<T: Clone> Clone for Slot<T> {
    fn clone(&self) -> Self {
        let (_new_sender, new_receiver) = std::sync::mpsc::channel(); // or std::sync::mpsc::channel()

        Self {
            sequence: self.sequence.clone(),
            receiver: Arc::new(Mutex::new(new_receiver)), 
        }
    }
}



impl<T> Slot<T>
where
    T: Send + 'static ,
{
    pub fn new(receiver: Receiver<T>, id_sequence : Option<usize>) -> Self {
        Slot {
            receiver: Arc::new(Mutex::new(receiver)),
            sequence: id_sequence.unwrap_or(0),
        }
    }

    pub fn start<F>(&mut self, handler: F)
    where
        F: FnMut(T) + Send + Sync + 'static,
    {
        let receiver = Arc::clone(&self.receiver);
        thread::spawn(move || {
            let mut handler = handler;
            let receiver = receiver.lock().unwrap();
            for command in receiver.iter() {
                handler(command);
            }
        });
    }
}

impl<T: Display> Display for Slot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot: {}", self.sequence)
    }
}


// impl Display for Slot<String> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Slot: {}", self.sequence)
//     }
// }

// impl Display for Slot<i32> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Slot: {}", self.sequence)
//     }
// }

// impl Display for Slot<f32> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "Slot: {}", self.sequence)
//     }
// }