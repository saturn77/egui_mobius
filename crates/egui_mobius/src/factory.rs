use std::sync::mpsc::{self, Sender, Receiver};
use crate::signals::Signal;
use crate::slot::Slot;
use std::sync::{Arc, Mutex, Condvar};
use std::fmt::Display; // for Slot<T> where T: Display
use std::time::Duration;

pub fn create_signal_slot<T>(id_sequence : usize) -> (Signal<T>, Slot<T>)
where
    T: Send + Clone + Display + 'static,
{
    let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();
    let signal = Signal::new(tx);
    let slot = Slot::new(rx, Some(id_sequence));
    (signal, slot)
}

/// A "dispatcher" thread spins up a vector of Slots, each of which is a receiver of messages.
/// It then sends a single message at a time to the main thread in order of sequence number.
/// Features : 
/// 1. Each producer has its own sequence number that is used to order messages
/// 2. Waits for messages to arrive in the rx_queue
/// 3. Sorts the messages by sequence number
/// 4. Forwards messages to the main thread in order
/// 5. Notifies the main thread when a message is forwarded
/// 6. Runs forever, i.e. it is "static" to the application
pub struct Dispatcher<T: 'static> {
    timeout       : Duration,
    rx_queue      : Arc<(Mutex<Vec<Slot<T>>>, Condvar)>,  // note that each Slot is an Arc<Mutex<Receiver<T>>>
    sequence      : usize,
    dispatcher_tx : Sender<T>,  // this is the channel to send messages to the main thread
}

impl<T: Send + std::fmt::Debug + 'static> Dispatcher<T> {
    /// Create a new Dispatcher
    pub fn new(dispatcher_tx: Sender<T>) -> Self {
        Dispatcher {
            timeout   : Duration::from_millis(100),
            rx_queue  : Arc::new((Mutex::new(Vec::new()), Condvar::new())),
            sequence  : 0,
            dispatcher_tx,
        }
    }

    /// Add a Slot to the dispatcher; a Slot may be driven by multiple Signals 
    pub fn add_slot(&mut self, slot: Slot<T>) {
        let (lock, _cvar) = &*self.rx_queue;
        let mut q = lock.lock().unwrap();
        q.push(slot);
    }

    /// Run the dispatcher thread, for the entire life time of the application
    pub fn run(&mut self) {
        loop {
            let (lock, cvar) = &*self.rx_queue;
            let mut q = lock.lock().unwrap();
            while q.is_empty() {
                let (new_q, timeout_result) = cvar.wait_timeout(q, self.timeout).unwrap();
                q = new_q;

                if timeout_result.timed_out() {
                    println!("[Dispatcher] Timeout occurred while waiting for messages.");
                    // Handle timeout case here (e.g., log a message, take other actions)
                }
            }
            q.sort_by_key(|slot| slot.sequence);
            while let Some(slot) = q.first() {
                if slot.sequence == self.sequence {
                    let msg = slot.receiver.lock().unwrap().recv().unwrap();
                    println!("[Dispatcher] Forwarding: {:?}", msg);
                    if let Err(e) = self.dispatcher_tx.send(msg) {
                        println!("***** Failed to send command: {:?}", e);
                    }
                    q.remove(0);
                    self.sequence += 1;
                } else {
                    break;
                }
            }
        }
    }
}
