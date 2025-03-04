use std::sync::mpsc::{self, Sender, Receiver};
use crate::signals::Signal;
use crate::slot::Slot;

pub fn create_signal_slot<T>() -> (Signal<T>, Slot<T>)
where
    T: Send + 'static,
{
    let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();
    let signal = Signal::new(tx);
    let slot = Slot::new(rx);
    (signal, slot)
}

// code below needs to be 'incorporated' into the code above


// use std::sync::{mpsc, Arc, Mutex, Condvar};
// use std::thread;
// use std::time::Duration;

// fn main() {
//     // Channel between dispatcher and main thread (final receiver)
//     let (dispatcher_tx, dispatcher_rx) = mpsc::channel();

//     // Shared queue with a condition variable for notification
//     let queue = Arc::new((Mutex::new(Vec::<(usize, String)>::new()), Condvar::new()));

//     // Spawn a dispatcher thread that serializes messages
//     let queue_clone = Arc::clone(&queue);
//     let dispatcher_handle = thread::spawn(move || {
//         let mut sequence = 0; // Global sequence counter

//         loop {
//             let (lock, cvar) = &*queue_clone;
//             let mut q = lock.lock().unwrap();

//             // Wait until there's at least one message in the queue
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
//             let queue_clone = Arc::clone(&queue);
//             thread::spawn(move || {
//                 for i in 0..5 {
//                     let msg = format!("Producer {} - Message {}", id, i);
//                     let (lock, cvar) = &*queue_clone;
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
