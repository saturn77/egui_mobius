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
    pub fn start<F>(&mut self, mut handler: F)
    where
        F: FnMut(T) + Send + 'static,
    {
        let receiver = Arc::clone(&self.receiver);
        thread::spawn(move || {
            let receiver = receiver.lock().unwrap();
            for event in receiver.iter() {
                handler(event);
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
    
    use egui_mobius_macros::EventMacro;

        /// Test with basic unit variants
        #[derive(EventMacro, PartialEq, Clone, Debug)]
        enum EventSimple {
            RefreshUI,
            CloseApp,
        }


    /// Define an enum for testing event processing
    #[derive(EventMacro, PartialEq, Clone, Debug)]
    enum Event {
        RefreshUI,
        UpdateData(String),
    }

    /// Define an enum for testing stateful event processing with FnMut 
    #[derive(Debug, PartialEq, Clone)]
    enum EventStateful {
        Increment,
        Decrement,
    }

    /// Define an enum for testing the EventMacro derive
    #[derive(EventMacro, PartialEq, Clone, Debug)]
    enum MyEventMacro {
        UpdateData(String),
    }

    #[test]
    /// Test the event handling functionality of the Slot struct.
    /// Create a channel for events, create a slot with the receiver, and start the slot in a separate thread.
    /// Send events and check if the events were processed correctly.
    /// This demonstrates the event handling functionality of the Slot struct.
    /// Also demonstrates how to use a single Signal/Slot pair to handle multiple types of events.
    fn test_event_handling() {
        // Create a channel for events.
        let (sender, receiver) = mpsc::channel();

        // Create a slot with the receiver.
        let mut slot = Slot::new(receiver, Some(1));

        // Shared storage to collect processed events.
        let processed_events = Arc::new(Mutex::new(Vec::new()));
        
        // Shadow the processed_events for the event handler into a checker for later
        // use in verifying the processed events.
        let events_checker = Arc::clone(&processed_events);

        // Start the slot with an event handler.
        slot.start(move |event: Event| {
            let processed_events = Arc::clone(&processed_events);
            let mut storage = processed_events.lock().unwrap();
            storage.push(event);
        });

        // Send events.
        sender.send(Event::RefreshUI).unwrap();
        sender.send(Event::UpdateData("New data".to_string())).unwrap();

        // Allow some time for the events to be processed.
        thread::sleep(Duration::from_millis(100));
        
        // Verify that the events were processed correctly.
        let events = events_checker.lock().unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0], Event::RefreshUI);
        assert_eq!(events[1], Event::UpdateData("New data".to_string()));
    }

    #[test]
    fn test_stateful_event_processing() {
        // Create a channel for events.
        let (sender, receiver) = mpsc::channel();

        // Create a slot with the receiver.
        let mut slot = Slot::new(receiver, Some(1));

        // Shared counter state.
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = Arc::clone(&counter);

        // Start the slot with a stateful handler.
        slot.start(move |event: EventStateful| {
            let mut count = counter_clone.lock().unwrap();
            match event {
                EventStateful::Increment => *count += 1,
                EventStateful::Decrement => *count -= 1,
            }
        });

        // Send events.
        sender.send(EventStateful::Increment).unwrap();
        sender.send(EventStateful::Increment).unwrap();
        sender.send(EventStateful::Decrement).unwrap();

        // Allow some time for the events to be processed.
        thread::sleep(Duration::from_millis(100));

        // Verify that the state has been updated correctly.
        let final_count = *counter.lock().unwrap();
        assert_eq!(final_count, 1);
    }

    /// test the EventMacro derive
    #[test]
    fn test_event_macro_derive() {

        let event = MyEventMacro::UpdateData("Hello!".to_string());

        println!("Event name: {}", event.event_name()); // Should print: UpdateData
    }

    /// Test with unit variants
    #[test]
    fn test_event_macro_unit_variants() {
        let event = EventSimple::RefreshUI;
        assert_eq!(event.event_name(), "RefreshUI");

        let event = EventSimple::CloseApp;
        assert_eq!(event.event_name(), "CloseApp");
    }

    /// Test with tuple variants
    #[derive(EventMacro, PartialEq, Clone, Debug)]
    enum EventTuple {
        UpdateData(String),
        ResizeWindow(u32, u32),
    }

    #[test]
    fn test_event_macro_tuple_variants() {
        let event = EventTuple::UpdateData("Hello".to_string());
        assert_eq!(event.event_name(), "UpdateData");

        let event = EventTuple::ResizeWindow(800, 600);
        assert_eq!(event.event_name(), "ResizeWindow");
    }

    /// Test with struct-like variants
    #[derive(EventMacro, PartialEq, Clone, Debug)]
    enum EventStruct {
        Custom { id: u32, name: String },
    }

    #[test]
    fn test_event_macro_struct_variants() {
        let event = EventStruct::Custom { id: 1, name: "Test".to_string() };
        assert_eq!(event.event_name(), "Custom");
    }

    /// Test multiple event types together
    #[derive(EventMacro, PartialEq, Clone, Debug)]
    enum EventMixed {
        Ping,
        Pong(u32),
        Data { key: String, value: i32 },
    }

    #[test]
    fn test_event_macro_mixed_variants() {
        assert_eq!(EventMixed::Ping.event_name(), "Ping");
        assert_eq!(EventMixed::Pong(42).event_name(), "Pong");
        assert_eq!(EventMixed::Data { key: "A".to_string(), value: 10 }.event_name(), "Data");
    }


}


