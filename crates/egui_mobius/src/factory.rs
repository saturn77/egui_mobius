use std::sync::mpsc::{self, Sender, Receiver};
use crate::signals::Signal;
use crate::slot::Slot;

use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Condvar};
use std::fmt::Debug;

/// Type alias for a priority queue of slots.
type SlotQueue<T> = PriorityQueue<Slot<T>, usize>;

/// Type alias for a synchronized queue with condition variable.
type SyncQueue<T> = Arc<(Mutex<SlotQueue<T>>, Condvar)>;

/// Creates a new signal-slot pair with the given sequence ID.
pub fn create_signal_slot<T>(id_sequence: usize) -> (Signal<T>, Slot<T>)
where
    T: Send + Clone + 'static,
{
    let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();
    let signal = Signal::new(tx);
    let slot = Slot::new(rx, Some(id_sequence));
    (signal, slot)
}

/// A dispatcher that manages multiple slots and forwards messages to the main thread in order of sequence number.
pub struct Dispatcher<T> {
    /// A map of slot names to slots.
    slots: HashMap<String, Slot<T>>,
    /// A priority queue to handle slots in order.
    rx_queue: SyncQueue<T>,
    /// The current sequence number.
    sequence: usize,
    /// A sender to the main thread.
    dispatcher_tx: Sender<T>,
}

impl<T> Dispatcher<T>
where
    T: Send + Debug + Clone + 'static,
{
    /// Creates a new dispatcher with the given sender to the main thread and a shutdown flag.
    pub fn new(dispatcher_tx: Sender<T>) -> Self {
        Self {
            slots: HashMap::new(),
            rx_queue: Arc::new((Mutex::new(PriorityQueue::new()), Condvar::new())),
            sequence: 0,
            dispatcher_tx,
        }
    }

    /// Adds a slot to the dispatcher with the given name.
    pub fn add_slot(&mut self, name: &str, slot: Slot<T>) {
        self.slots.insert(name.to_string(), slot);
    }

    /// Sets a handler for the slot with the given name.
    ///
    /// The handler is a function that will be called with the message received by the slot.
    pub fn set_handler<F>(&mut self, slot_name: &str, handler: F)
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        if let Some(slot) = self.slots.get_mut(slot_name) {
            slot.start(handler);
        } else {
            println!("Error: Slot '{}' not found!", slot_name);
        }
    }

    /// Runs the dispatcher, processing messages from the slots in order of their sequence numbers.
    ///
    /// This method runs indefinitely, waiting for messages to arrive in the queue and forwarding them to the main thread.
    pub fn run(&mut self) {
        loop {

            let (lock, cvar) = &*self.rx_queue;
            let mut q = lock.lock().unwrap();
            while q.is_empty() {
                q = cvar.wait(q).unwrap();
            }
            while let Some((slot, priority)) = q.peek() {
                if *priority == self.sequence {
                    let msg = slot.receiver.lock().unwrap().recv().unwrap();
                    println!("[Dispatcher] Forwarding: {:?}", msg);
                    if let Err(e) = self.dispatcher_tx.send(msg) {
                        println!("***** Failed to send command: {:?}", e);
                    }
                    q.pop();
                    self.sequence += 1;
                } else {
                    break;
                }
            }
        }
    }
}
