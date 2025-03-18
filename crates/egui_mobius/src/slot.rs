//! The Slot module provides a self-contained threaded channel receiver.
//! 
//! Slot is a core component of the egui_mobius library that handles message processing
//! in its own dedicated thread. It can receive messages of type Event, Command,
//! Response, or any other type that implements `T: Send + 'static + Clone`.
//! 
//! Each Slot maintains its own thread for message processing, allowing concurrent
//! execution independent of the main application thread. Messages are processed by
//! a user-provided handler function that defines the slot's behavior.
//!

use std::sync::{Arc, Mutex};
use std::fmt::{Debug, Display};
use std::sync::mpsc::Receiver;
use std::thread;

/// Slot struct with receiver and sequence number.
///
/// Slot is a primary component of the egui_mobius library, 
/// and slots are used to receive messages that may be of
/// an Event, Command, Response, or any other type.
pub struct Slot<T> {
    pub receiver: Arc<Mutex<Receiver<T>>>,
}

//-----------------Rust Trait Implementations ---------------------//

/// Implementations for `Slot<T>` of the following traits:
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
        let (_new_sender, new_receiver) = std::sync::mpsc::channel();
        Self {
            receiver: Arc::new(Mutex::new(new_receiver)), 
        }
    }
}

/// Display for ```Slot<T>```
impl<T: Display> Display for Slot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot")
    }
}

/// Debug for ```Slot<T>```
impl <T: Debug> Debug for Slot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot")
    }
}

//------------------ Slot Implementations ---------------------//

/// Slot implementation. Note that Slot implements new for a 
/// type T that is Send, 'static, and Clone.
impl<T> Slot<T>
where
    T: Send + 'static + Clone,
{
    /// Create a new slot with the given receiver and sequence ID.
    pub fn new(receiver: Receiver<T>) -> Self {
        Slot {
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    /// Start the slot, which means giving the Slot its own thread to process messages.
    /// This is a key component of egui_mobius, as it allows for the Slot to process messages
    /// in a separate thread from the main application.
    pub fn start<F>(&mut self, mut handler: F)
    where
        F: FnMut(T) + Send + 'static,
    {
        let receiver = Arc::clone(&self.receiver);
        thread::spawn(move || {
            let receiver = receiver.lock().unwrap();
            for msg_or_event in receiver.iter() {
                handler(msg_or_event);
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
    
    /// Test with basic unit variants
    #[derive(PartialEq, Clone, Debug)]
    enum EventSimple {
        RefreshUI,
        CloseApp,
    }

    /// Define an enum for testing event processing
    #[derive(PartialEq, Clone, Debug)]
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

    /// Define an enum for testing event processing
    #[derive(PartialEq, Clone, Debug)]
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
        let mut slot = Slot::new(receiver);

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
        let mut slot = Slot::new(receiver);

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
    /// Test different event enum variants that will be used with future derive macros
    #[derive(PartialEq, Clone, Debug)]
    enum EventTuple {
        UpdateData(String),
        ResizeWindow(u32, u32),
    }

    #[derive(PartialEq, Clone, Debug)]
    enum EventStruct {
        Custom { id: u32, name: String },
    }

    #[derive(PartialEq, Clone, Debug)]
    enum EventMixed {
        Ping,
        Pong(u32),
        Data { key: String, value: i32 },
    }

    #[test]
    fn test_event_types() {
        // Test that we can create and use different event types
        let _event = MyEventMacro::UpdateData("test".to_string());
        
        let _simple = EventSimple::RefreshUI;
        let _simple_alt = EventSimple::CloseApp;
        
        let _tuple = EventTuple::UpdateData("test".to_string());
        let _tuple_alt = EventTuple::ResizeWindow(800, 600);
        
        let _struct = EventStruct::Custom { id: 1, name: "test".to_string() };
        
        let _mixed = EventMixed::Ping;
        let _mixed_tuple = EventMixed::Pong(42);
        let _mixed_struct = EventMixed::Data { key: "A".to_string(), value: 10 };
    }


}


