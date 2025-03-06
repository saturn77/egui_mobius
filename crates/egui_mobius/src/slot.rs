use std::sync::{Arc, Mutex};
use std::fmt::{Debug, Display};
use std::sync::mpsc::Receiver;
use std::thread;
use std::cmp::Ordering;

/// Slot struct with receiver and sequence number.
pub struct Slot<T> {
    pub receiver: Arc<Mutex<Receiver<T>>>,
    pub sequence: usize,
}



//-----------------Rust Trait Implementations ---------------------//

/// Impelmentations for Slot<T> of the following traits:
/// - Clone
/// - Hash
/// - PartialEq
/// - Eq
/// - PartialOrd
/// - Ord
/// - Display
/// - Debug
impl<T : Clone> Clone for Slot<T> {
    fn clone(&self) -> Self {
        let (_new_sender, new_receiver) = std::sync::mpsc::channel(); // or std::sync::mpsc::channel()

        Self {
            sequence: self.sequence.clone(),
            receiver: Arc::new(Mutex::new(new_receiver)), 
        }
    }
}

/// Hash
impl<T> std::hash::Hash for Slot<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.sequence.hash(state);
    }
}

/// PartialEq, Eq, PartialOrd, Ord
impl<T> PartialEq for Slot<T> {
    fn eq(&self, other: &Self) -> bool {
        self.sequence == other.sequence
    }
}

/// Eq
impl<T> Eq for Slot<T> {}

/// PartialOrd
impl<T> PartialOrd for Slot<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Ord
impl<T> Ord for Slot<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sequence.cmp(&other.sequence)
    }
}

/// Display 
impl<T: Display> Display for Slot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot: {}", self.sequence)
    }
}

/// Debug
impl <T: Debug> Debug for Slot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot: {}", self.sequence)
    }
}

//------------------ Slot Implementations ---------------------//

/// Slot implementation. The Slot struct is used to receive commands from the MobiusEnque<Command> sender.
/// The start method is used to start the slot in a separate thread.
/// Integration of Slots with the Dispatcher is done in the Dispatcher implementation.
/// The Dispatcher will manage multiple slots and forward messages to the main thread in order of sequence number.
impl<T> Slot<T>
where
    T: Send + 'static + Clone,
{
    /// Create a new slot with the given receiver and sequence ID.
    pub fn new(receiver: Receiver<T>, id_sequence : Option<usize>) -> Self {
        Slot {
            receiver: Arc::new(Mutex::new(receiver)),
            sequence: id_sequence.unwrap_or(0),
        }
    }

    /// Start the slot in a separate thread.
    pub fn start<F>(&mut self, handler: F)
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        let receiver = Arc::clone(&self.receiver);
        thread::spawn(move || {
            let handler = handler;
            let receiver = receiver.lock().unwrap();
            for command in receiver.iter() {
                handler(command);
            }
        });
    }
}
