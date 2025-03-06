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


#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{mpsc, Arc, Mutex};
    use std::time::Duration;
    use std::thread;

    #[derive(Debug, PartialEq, Clone)]
    enum Event {
        RefreshUI,
        UpdateData(String),
    }

    #[test]
    /// Test the Slot signal across threads.
    /// Create a channel (Signal/Slot pair), create a slot with the receiver, and start the slot in a separate thread.
    /// Send a test message and check if the message was received and processed.
    /// This demonstrates fundamental functionality of the Slot struct.
    fn test_slot_signal_across_thread() {
        // Create a channel (Signal/Slot pair)
        let (sender, receiver) = mpsc::channel();

        // Create a slot with the receiver
        let mut slot = Slot::new(receiver, Some(1));

        // Shared storage to collect processed messages
        let received_messages = Arc::new(Mutex::new(Vec::new()));
        let received_messages_clone = Arc::clone(&received_messages);

        // Start the slot in a separate thread
        slot.start(move |msg: String| {
            let mut storage = received_messages_clone.lock().unwrap();
            storage.push(msg);
        });

        // Send a test message
        sender.send("Hello from the other thread!".to_string()).unwrap();

        // Allow some time for the thread to process
        thread::sleep(Duration::from_millis(100));

        // Check if the message was received and processed
        let messages = received_messages.lock().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], "Hello from the other thread!");
    }

    #[test]
    /// Test the event handling functionality of the Slot struct.
    /// Create a channel for events, create a slot with the receiver, and start the slot in a separate thread.
    /// Send events and check if the events were processed correctly.
    /// This demonstrates the event handling functionality of the Slot struct.
    /// Also demonstrates how to use a single Signal/Slot pair to handle multiple types of events.
    fn test_event_handling() {
        // Create a channel for events.
        let (sender, mut receiver) = mpsc::channel();

        // Create a slot with the receiver.
        let mut slot = Slot::new(receiver, Some(1));

        // Shared storage to collect processed events.
        let processed_events = Arc::new(Mutex::new(Vec::new()));
        let processed_events_clone = Arc::clone(&processed_events);

        // Start the slot with an event handler.
        slot.start(move |event: Event| {
            let mut storage = processed_events_clone.lock().unwrap();
            storage.push(event);
        });

        // Send events.
        sender.send(Event::RefreshUI).unwrap();
        sender.send(Event::UpdateData("New data".to_string())).unwrap();

        // Allow some time for the events to be processed.
        thread::sleep(Duration::from_millis(100));

        // Verify that the events were processed correctly.
        let events = processed_events.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], Event::RefreshUI);
        assert_eq!(events[1], Event::UpdateData("New data".to_string()));
    }
}


