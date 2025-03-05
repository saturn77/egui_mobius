use std::sync::mpsc::{self, Sender, Receiver};
use crate::signals::Signal;
use crate::slot::Slot;
use std::sync::{Arc, Mutex, Condvar};
use std::fmt::Display; // for Slot<T> where T: Display
use std::fmt::Debug; // for Signal<T> where T: Debug
use std::time::Duration;

pub fn create_signal_slot<T>() -> (Signal<T>, Slot<T>)
where
    T: Send + Clone + Debug + Display + 'static,
{
    let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();
    let signal = Signal::new(tx);
    let slot = Slot::new(rx);
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
struct Dispatcher<T: Clone + 'static> {
    timeout       : Duration,
    rx_queue      : Arc<(Mutex<Vec<Slot<T>>>, Condvar)>,  // note that each Slot is an Arc<Mutex<Receiver<T>>>
    sequence      : usize,
    tx_dispatcher : Signal<T>,  // this is the Signal that the dispatcher uses to send messages to the main thread
}

impl<T: Send + Clone + Debug + Display + 'static> Dispatcher<T> {
    /// Create a new Dispatcher
    pub fn new() -> Self {
        Dispatcher {
            timeout   : Duration::from_millis(100),
            rx_queue  : Arc::new((Mutex::new(Vec::new()), Condvar::new())),
            sequence  : 0,
            tx_dispatcher : {
                let (tx, _rx) = create_signal_slot::<T>();
                tx
            },
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
                    self.tx_dispatcher.send(msg).unwrap();
                    q.remove(0);
                    self.sequence += 1;
                } else {
                    break;
                }
            }
        }
    }
}






// code below needs to be 'incorporated' into the code above


// use std::sync::{mpsc, Arc, Mutex, Condvar};
// use std::thread;
// use std::time::Duration;

// fn main() {
//     // Channel between dispatcher and main thread (final receiver)
//     let (dispatcher_tx, dispatcher_rx) = mpsc::channel();

//     // Shared rx_queue with a condition variable for notification
//     let rx_queue = Arc::new((Mutex::new(Vec::<(usize, String)>::new()), Condvar::new()));

//     // Spawn a dispatcher thread that serializes messages
//     let rx_queue_clone = Arc::clone(&rx_queue);
//     let dispatcher_handle = thread::spawn(move || {
//         let mut sequence = 0; // Global sequence counter

//         loop {
//             let (lock, cvar) = &*rx_queue_clone;
//             let mut q = lock.lock().unwrap();

//             // Wait until there's at least one message in the rx_queue
//             while q.is_empty() {
//                 q = cvar.wait(q).unwrap();
//             }

//             // Sort by assigned sequence number
//             q.sort_by_key(|&(seq, _)| seq);
//             while let Some((seq, msg)) = q.first().cloned() {
//                 if seq == sequence {
//                     println!("[Dispatcher] Forwarding: {}", msg);
//                     dispatcher_tx.send(msg).unwrap();
//                     q.remove(0);
//                     sequence += 1;
//                 } else {
//                     break;
//                 }
//             }
//         }
//     });

//     // Simulate multiple producers
//     let producers: Vec<_> = (0..3)
//         .map(|id| {
//             let rx_queue_clone = Arc::clone(&rx_queue);
//             thread::spawn(move || {
//                 for i in 0..5 {
//                     let msg = format!("Producer {} - Message {}", id, i);
//                     let (lock, cvar) = &*rx_queue_clone;
//                     let mut q = lock.lock().unwrap();
//                     let seq = q.len(); // Assign a sequence number (local per producer)
//                     q.push((seq, msg));
//                     cvar.notify_one(); // Wake up dispatcher
//                     drop(q);
//                     thread::sleep(Duration::from_millis(50)); // Simulate work
//                 }
//             })
//         })
//         .collect();

//     // Receive messages in order
//     for received in dispatcher_rx {
//         println!("[Main] Received: {}", received);
//     }

//     // Join all producer threads
//     for p in producers {
//         p.join().unwrap();
//     }

//     // Join dispatcher thread (never-ending loop here, adjust as needed)
//     dispatcher_handle.join().unwrap();
// }
