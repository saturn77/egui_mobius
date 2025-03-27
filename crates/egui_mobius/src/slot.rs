//! The Slot module provides a self-contained threaded or async channel receiver.
//!
//! Slot is a core component of the egui_mobius library that handles message processing
//! either in its own dedicated thread or via the async tokio executor. It can receive
//! messages of type Event, Command, Response, or any other type that implements
//! `T: Send + 'static + Clone`.
//!
//! Each Slot can run on its own thread or within the tokio runtime, allowing flexible
//! concurrent execution independent of the main application thread.

use std::sync::{Arc, Mutex};
use std::fmt::{Debug, Display};
use std::sync::mpsc::Receiver;
use std::thread;
use std::panic::AssertUnwindSafe;
use futures::FutureExt;

/// Slot struct with receiver
pub struct Slot<T> {
    pub receiver: Arc<Mutex<Receiver<T>>>,
}

impl<T: Clone> Clone for Slot<T> {
    fn clone(&self) -> Self {
        let (_new_sender, new_receiver) = std::sync::mpsc::channel();
        Self {
            receiver: Arc::new(Mutex::new(new_receiver)),
        }
    }
}

impl<T: Display> Display for Slot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot")
    }
}

impl<T: Debug> Debug for Slot<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Slot")
    }
}

impl<T> Slot<T>
where
    T: Send + 'static + Clone,
{
    pub fn new(receiver: Receiver<T>) -> Self {
        Slot {
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }

    /// Start the slot using a dedicated thread.
    pub fn start<F>(&mut self, mut handler: F)
    where
        F: FnMut(T) + Send + 'static,
    {
        let receiver = Arc::clone(&self.receiver);
        thread::spawn(move || {
            let receiver = receiver.lock().unwrap();
            for msg in receiver.iter() {
                handler(msg);
            }
        });
    }

    /// Start the slot using an async handler with tokio executor.
    pub fn start_async<F, Fut>(&mut self, mut handler: F)
    where
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let receiver = Arc::clone(&self.receiver);
        tokio::spawn(async move {
            loop {
                let msg = {
                    let guard = receiver.lock().unwrap();
                    guard.try_recv().ok() // Simplified using `.ok()`
                };

                if let Some(msg) = msg {
                    let fut = handler(msg);
                    tokio::spawn(async move {
                        if let Err(err) = AssertUnwindSafe(fut).catch_unwind().await {
                            eprintln!("⚠️  async handler panicked: {:?}", err);
                        }
                    });
                }

                // Give other tasks a chance to run
                tokio::task::yield_now().await;
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
    use tokio::sync::Notify;

    #[derive(PartialEq, Clone, Debug)]
    enum Event {
        Add(u32),
        Sub(u32),
    }

    #[test]
    fn test_threaded_slot() {
        let (sender, receiver) = mpsc::channel();
        let mut slot = Slot::new(receiver);
        let result = Arc::new(Mutex::new(0));
        let result_clone = Arc::clone(&result);

        slot.start(move |event: Event| {
            let mut val = result_clone.lock().unwrap();
            match event {
                Event::Add(x) => *val += x,
                Event::Sub(x) => *val -= x,
            }
        });

        sender.send(Event::Add(5)).unwrap();
        sender.send(Event::Sub(2)).unwrap();
        thread::sleep(Duration::from_millis(100));

        let final_val = *result.lock().unwrap();
        assert_eq!(final_val, 3);
    }

    #[tokio::test]
    async fn test_async_slot_tokio_single_message() {
        let (sender, receiver) = mpsc::channel();
        let mut slot = Slot::new(receiver);
        let result = Arc::new(Mutex::new(0));
        let result_clone = Arc::clone(&result);
        let notify = Arc::new(Notify::new());
        let notify_clone = notify.clone();

        slot.start_async(move |event: Event| {
            let result_clone = Arc::clone(&result_clone);
            let notify_clone = notify_clone.clone();
            async move {
                let mut val = result_clone.lock().unwrap();
                match event {
                    Event::Add(x) => *val += x,
                    Event::Sub(x) => *val -= x,
                }
                notify_clone.notify_one();
            }
        });

        sender.send(Event::Add(10)).unwrap();
        notify.notified().await;

        let final_val = *result.lock().unwrap();
        assert_eq!(final_val, 10);
    }

    #[tokio::test]
    async fn test_async_slot_tokio_multiple_messages() {
        let (sender, receiver) = mpsc::channel();
        let mut slot = Slot::new(receiver);
        let result = Arc::new(Mutex::new(100));
        let result_clone = Arc::clone(&result);

        slot.start_async(move |event: Event| {
            let result_clone = Arc::clone(&result_clone);
            async move {
                let mut val = result_clone.lock().unwrap();
                match event {
                    Event::Add(x) => *val += x,
                    Event::Sub(x) => *val -= x,
                }
            }
        });

        sender.send(Event::Add(50)).unwrap();
        sender.send(Event::Sub(20)).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        let final_val = *result.lock().unwrap();
        assert_eq!(final_val, 130);
    }

    #[tokio::test]
    async fn test_async_slot_empty_queue() {
        let (_sender, receiver) = mpsc::channel();
        let mut slot = Slot::new(receiver);

        slot.start_async(move |_event: Event| async move {
            panic!("Should not be called");
        });

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    #[tokio::test]
    async fn test_async_slot_handler_panics() {
        let (sender, receiver) = mpsc::channel();
        let mut slot = Slot::new(receiver);
        let result = Arc::new(Mutex::new(0));
        let result_clone = Arc::clone(&result);

        slot.start_async(move |event: Event| {
            let result_clone = Arc::clone(&result_clone);
            async move {
                if let Event::Add(999) = event {
                    panic!("Simulated handler panic");
                }
                let mut val = result_clone.lock().unwrap();
                *val += 1;
            }
        });

        let _ = sender.send(Event::Add(999));
        let _ = sender.send(Event::Add(1));
        tokio::time::sleep(Duration::from_millis(100)).await;

        let val = *result.lock().unwrap();
        assert_eq!(val, 1); // Only one increment should succeed
    }

    #[tokio::test]
    async fn test_multiple_async_slots_run_independently() {
        let (sender1, receiver1) = mpsc::channel();
        let (sender2, receiver2) = mpsc::channel();
        let mut slot1 = Slot::new(receiver1);
        let mut slot2 = Slot::new(receiver2);

        let res1 = Arc::new(Mutex::new(0));
        let res2 = Arc::new(Mutex::new(0));

        let res1_clone = Arc::clone(&res1);
        let res2_clone = Arc::clone(&res2);

        slot1.start_async(move |event| {
            let res1 = Arc::clone(&res1_clone);
            async move {
                if let Event::Add(x) = event {
                    let mut val = res1.lock().unwrap();
                    *val += x;
                }
            }
        });

        slot2.start_async(move |event| {
            let res2 = Arc::clone(&res2_clone);
            async move {
                if let Event::Sub(x) = event {
                    let mut val = res2.lock().unwrap();
                    *val += x;
                }
            }
        });

        sender1.send(Event::Add(3)).unwrap();
        sender2.send(Event::Sub(7)).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;

        assert_eq!(*res1.lock().unwrap(), 3);
        assert_eq!(*res2.lock().unwrap(), 7);
    }
}
