use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::collections::VecDeque;

// Atomic counter for strict message ordering
static LAST_ORDER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

// Struct to store messages with an order
#[derive(Debug)]
struct TimedMessage {
    order: usize,
    content: String,
}

fn main() {
    println!("\n**** Final Ordered Dispatcher Example ****\n");

    // Create a thread-safe queue (VecDeque inside a Mutex)
    let message_queue = Arc::new(Mutex::new(VecDeque::new()));

    // Message channel (for inter-thread communication)
    let (tx, rx) = mpsc::sync_channel(100); // Use sync_channel to prevent interleaving

    // Producer Thread: Ensures messages are enqueued in the correct order
    let queue_clone = Arc::clone(&message_queue);
    let tx_clone = tx.clone();
    thread::spawn(move || {
        for i in 0..10 {
            thread::sleep(Duration::from_secs(1)); // Simulate delay between messages
            
            // Ensure strict ordering
            let order = LAST_ORDER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            // Add Foo - Egui first
            let message1 = TimedMessage {
                order,
                content: format!("Handler 1 received: Foo - Egui - {}", i),
            };
            queue_clone.lock().unwrap().push_back(message1);

            // Add Bar - Mobius second
            let message2 = TimedMessage {
                order: order + 1, // Ensuring it's strictly after Foo - Egui
                content: format!("Handler 2 received: Bar - Mobius - {}", i),
            };
            queue_clone.lock().unwrap().push_back(message2);

            tx_clone.send(()).unwrap(); // Signal processing thread
        }
    });

    // Processing Thread: Ensures ordered message output
    let queue_clone2 = Arc::clone(&message_queue);
    thread::spawn(move || {
        loop {
            if rx.recv().is_err() {
                break;
            }

            let mut queue = queue_clone2.lock().unwrap();

            // Sort messages strictly by order before printing
            let mut sorted_messages: Vec<TimedMessage> = queue.drain(..).collect();
            sorted_messages.sort_by_key(|m| m.order);

            // Print messages in strict order
            for msg in sorted_messages {
                println!("{}", msg.content);

                // Add a newline after Mobius messages
                if msg.content.contains("Mobius") {
                    println!();
                }

                // Stop application after last expected message
                if msg.content == "Handler 2 received: Bar - Mobius - 9" {
                    println!("\nShutting down now, because the last message was received.");
                    println!("Hope you enjoyed the Dispatcher -- Goodbye!");
                    std::process::exit(0);
                }
            }
        }
    });

    // Keep the main thread alive to allow messages to be processed
    loop {
        thread::sleep(Duration::from_secs(1));
    }
}
