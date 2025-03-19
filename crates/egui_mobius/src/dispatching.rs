//! Dispatcher module for egui_mobius
//!
//! This module provides a generic signal-slot system using `Dispatcher` and `SignalDispatcher`.
//! It supports named channels for message routing and integrates seamlessly with the `Value<T>` type
//! from `egui_mobius`. The dispatcher system enables decoupled communication between components
//! through a type-safe event system.
//!
//! ## Example
//!
//! ```rust
//! use egui_mobius::dispatching::{Dispatcher, SignalDispatcher};
//!
//! #[derive(Clone)]
//! enum Event {
//!     Hello,
//!     Text(String),
//! }
//!
//! let dispatcher = Dispatcher::<Event>::default();
//!
//! dispatcher.register_slot("greet", |event| {
//!     if let Event::Text(text) = event {
//!         println!("Received: {}", text);
//!     }
//! });
//!
//! dispatcher.send("greet", Event::Text("hi from egui_mobius".into()));
//! ```

use std::collections::HashMap;
use crate::types::Value;
use crate::slot::Slot;
use crate::signals::Signal;
use std::future::Future;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Type alias for a handler function that can process events.
type HandlerFn<E> = dyn Fn(E) + Send + Sync;

/// Type alias for a collection of event handlers.
type HandlerMap<E> = HashMap<String, Vec<Arc<HandlerFn<E>>>>;


/// The `SignalDispatcher` trait provides a generic interface
/// for sending and receiving typed events across named channels.
/// A trait representing a generic dispatcher capable of sending events to
/// slots (handlers) registered on named channels.
///
/// This trait allows you to decouple event producers from consumers in
/// `egui_mobius` applications using a simple signal-slot architecture.
///
/// # Type Parameters
/// - `E`: The event type this dispatcher works with.
///
/// # Example
/// ```rust
/// use egui_mobius::dispatching::Dispatcher;
///
/// #[derive(Clone)]
/// enum MyEvent {
///     Something,
/// }
///
/// let dispatcher = Dispatcher::<MyEvent>::default();
/// ```
pub trait SignalDispatcher<E> {
    fn send(&self, channel: &str, event: E);

    /// Register a slot (event handler) for a specific named channel.
    /// Multiple slots can be registered per channel.
    ///
    /// # Parameters
    /// - `channel`: the name of the channel to listen to
    /// - `f`: closure that will be called with each event
    fn register_slot<F>(&self, channel: &str, f: F)
    where
        F: Fn(E) + Send + Sync + 'static;
}

/// A generic event dispatcher for a given event type `E`.
/// Stores handlers (slots) for named channels and dispatches events to them.
#[derive(Clone)]
pub struct Dispatcher<E> {
    handlers: Value<HandlerMap<E>>,
}

impl<E: Clone + Send + 'static> Default for Dispatcher<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Clone + Send + 'static> Dispatcher<E> {
    /// Create a new, empty `Dispatcher` instance.
    ///
    /// Typically stored inside an `egui_mobius::Value` or `Arc`.
    ///
    /// # Example
    /// ```rust
    /// #[derive(Clone)]
    /// enum MyEvent {
    ///     Something,
    /// }
    /// use egui_mobius::dispatching::Dispatcher;
    /// let dispatcher = Dispatcher::<MyEvent>::default();
    /// ```
    pub fn new() -> Self {
        Self {
            handlers: Value::new(HashMap::new()),
        }
    }
}

impl<E: Clone + Send + 'static> SignalDispatcher<E> for Dispatcher<E> {
    /// Send an event to all handlers registered for the given channel.
    ///
    /// If no slots are registered on the channel, this is a no-op.
    ///
    /// # Parameters
    /// - `channel`: name of the logical channel
    /// - `event`: event value to be dispatched
    fn send(&self, channel: &str, event: E) {
        if let Some(slots) = self.handlers.get().get(channel) {
            for handler in slots {
                handler(event.clone());
            }
        }
    }

    fn register_slot<F>(&self, channel: &str, f: F)
    where
        F: Fn(E) + Send + Sync + 'static,
    {
        let mut map = self.handlers.lock().unwrap();
        map.entry(channel.to_string())
            .or_default()
            .push(std::sync::Arc::new(f));
    }
}




/// An asynchronous dispatcher that processes events in a dedicated thread pool and
/// supports non-blocking operations with proper error handling and timeouts.
///
/// The `AsyncDispatcher` is particularly useful for:
/// - Long-running background tasks that shouldn't block the UI
/// - Network requests and file I/O operations
/// - Parallel processing of computationally intensive tasks
/// - Operations that require timeouts or cancellation
///
/// # Type Parameters
/// - `E`: The event type this dispatcher processes
/// - `R`: The result type returned after processing
///
/// # Examples
///
/// ## Basic Usage
/// ```rust
/// use egui_mobius::dispatching::AsyncDispatcher;
/// use egui_mobius::factory::create_signal_slot;
///
/// // Create an async dispatcher for processing image data
/// let dispatcher = AsyncDispatcher::<Vec<u8>, String>::new();
///
/// // Set up signal/slot for image processing
/// let (signal, slot) = create_signal_slot::<Vec<u8>>();
/// let (result_signal, result_slot) = create_signal_slot::<String>();
///
/// // Attach async handler
/// dispatcher.attach_async(slot, result_signal, |image_data| async move {
///     // Simulate image processing
///     format!("Processed {} bytes of image data", image_data.len())
/// });
/// ```
///
/// ## With Timeouts and Error Handling
/// ```rust
/// use egui_mobius::dispatching::AsyncDispatcher;
/// use egui_mobius::factory::create_signal_slot;
/// use std::time::Duration;
/// use tokio::time::timeout;
///
/// #[derive(Debug, Clone)]
/// enum ProcessError {
///     Timeout,
///     Failed(String),
/// }
///
/// let dispatcher = AsyncDispatcher::<String, Result<String, ProcessError>>::new();
/// let (signal, slot) = create_signal_slot::<String>();
/// let (result_signal, result_slot) = create_signal_slot::<Result<String, ProcessError>>();
///
/// dispatcher.attach_async(slot, result_signal, |input| async move {
///     match timeout(Duration::from_secs(5), async {
///         // Simulate long-running task
///         tokio::time::sleep(Duration::from_secs(1)).await;
///         Ok(format!("Processed: {}", input))
///     }).await {
///         Ok(result) => result,
///         Err(_) => Err(ProcessError::Timeout),
///     }
/// });
/// ```
///
/// ## Parallel Processing
/// ```rust
/// use egui_mobius::dispatching::AsyncDispatcher;
/// use egui_mobius::factory::create_signal_slot;
/// use futures::future::join_all;
///
/// let dispatcher = AsyncDispatcher::<Vec<i32>, Vec<i32>>::new();
/// let (signal, slot) = create_signal_slot::<Vec<i32>>();
/// let (result_signal, result_slot) = create_signal_slot::<Vec<i32>>();
///
/// dispatcher.attach_async(slot, result_signal, |numbers| async move {
///     let tasks: Vec<_> = numbers.into_iter()
///         .map(|n| async move { n * n })
///         .collect();
///     join_all(tasks).await
/// });
/// ```
pub struct AsyncDispatcher<E, R> {
    runtime: Arc<Runtime>,
    _phantom: std::marker::PhantomData<(E, R)>,
}

impl<E: Send + 'static, R: Send + 'static> Default for AsyncDispatcher<E, R> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Send + 'static, R: Send + 'static> AsyncDispatcher<E, R> {
    /// Creates a new `AsyncDispatcher` with its own Tokio runtime.
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to build Tokio runtime");
        Self {
            runtime: Arc::new(runtime),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Attaches an async handler to the given `Slot<E>`, processing events asynchronously
    /// and sending results via `Signal<R>`. The handler runs in a dedicated thread pool
    /// managed by Tokio, ensuring non-blocking operation.
    ///
    /// # Arguments
    /// * `slot` - The slot that will receive events to process
    /// * `signal` - The signal used to send processed results
    /// * `handler` - An async closure that processes events and returns results
    ///
    /// # Type Parameters
    /// * `F` - The handler function type that takes an event and returns a Future
    /// * `Fut` - The Future type returned by the handler
    ///
    /// # Notes
    /// - This can be called only once per Slot
    /// - The handler runs in a Tokio runtime with work-stealing scheduler
    /// - Results are sent asynchronously through the signal
    /// - If the signal send fails (e.g., no receivers), the error is silently ignored
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius::dispatching::AsyncDispatcher;
    /// use egui_mobius::factory::create_signal_slot;
    /// use tokio::time::sleep;
    /// use std::time::Duration;
    ///
    /// async fn process_data(data: String) -> Result<String, String> {
    ///     sleep(Duration::from_millis(100)).await; // Simulate work
    ///     Ok(format!("Processed: {}", data))
    /// }
    ///
    /// let dispatcher = AsyncDispatcher::<String, Result<String, String>>::new();
    /// let (signal, slot) = create_signal_slot::<String>();
    /// let (result_signal, result_slot) = create_signal_slot::<Result<String, String>>();
    ///
    /// dispatcher.attach_async(slot, result_signal, |input| async move {
    ///     process_data(input).await
    /// });
    /// ```
    pub fn attach_async<F, Fut>(
        &self,
        mut slot: Slot<E>,
        signal: Signal<R>,
        handler: F,
    ) where
        E: Clone + Send + 'static,
        R: Send + 'static,
        F: Fn(E) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
    {
        let runtime = self.runtime.clone();
        let handler = Arc::new(handler); // satisfy Fn(E) + Send + Sync
    
        slot.start({
            let handler = handler.clone();
            move |event| {
                let fut = handler(event);
                let signal = signal.clone();
                runtime.spawn(async move {
                    let result = fut.await;
                    let _ = signal.send(result);
                });
            }
        });
    }
    
    
}



#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum TestEvent {
        Ping,
        Message(String),
    }

    #[test]
    fn dispatcher_should_invoke_slot_on_send() {
        let dispatcher = Dispatcher::<TestEvent>::new();

        let called = std::sync::Arc::new(std::sync::Mutex::new(false));
        let called_clone = called.clone();

        dispatcher.register_slot("test", move |event| {
            if let TestEvent::Ping = event {
                *called_clone.lock().unwrap() = true;
            }
        });

        dispatcher.send("test", TestEvent::Ping);
        assert_eq!(*called.lock().unwrap(), true);
    }

    #[test]
    fn dispatcher_supports_multiple_slots_per_channel() {
        let dispatcher = Dispatcher::<TestEvent>::new();
        let results_ref = std::sync::Arc::new(std::sync::Mutex::new(vec![]));

        for i in 0..3 {
            let results_clone = results_ref.clone();
            dispatcher.register_slot("log", move |event| {
                if let TestEvent::Message(msg) = event {
                    results_clone.lock().unwrap().push((i, msg));
                }
            });
        }

        dispatcher.send("log", TestEvent::Message("Hello".into()));

        let collected = results_ref.lock().unwrap();
        assert_eq!(collected.len(), 3);

        let mut ids = collected.iter().map(|(id, _)| *id).collect::<Vec<_>>();
        ids.sort();
        assert_eq!(ids, vec![0, 1, 2]);

        for (_, msg) in collected.iter() {
            assert_eq!(msg, "Hello");
        }
    }

    #[test]
    fn dispatcher_handles_multiple_channels_independently() {
        let dispatcher = Dispatcher::<TestEvent>::new();

        let alpha_flag = std::sync::Arc::new(std::sync::Mutex::new(false));
        let beta_flag = std::sync::Arc::new(std::sync::Mutex::new(false));

        let alpha_clone = alpha_flag.clone();
        let beta_clone = beta_flag.clone();

        dispatcher.register_slot("alpha", move |event| {
            if let TestEvent::Message(msg) = event {
                if msg == "alpha" {
                    *alpha_clone.lock().unwrap() = true;
                }
            }
        });

        dispatcher.register_slot("beta", move |event| {
            if let TestEvent::Message(msg) = event {
                if msg == "beta" {
                    *beta_clone.lock().unwrap() = true;
                }
            }
        });

        dispatcher.send("alpha", TestEvent::Message("alpha".into()));
        dispatcher.send("beta", TestEvent::Message("beta".into()));

        assert_eq!(*alpha_flag.lock().unwrap(), true);
        assert_eq!(*beta_flag.lock().unwrap(), true);
    }

    #[test]
    fn dispatcher_send_to_unregistered_channel_does_nothing() {
        let dispatcher = Dispatcher::<TestEvent>::new();
        dispatcher.send("unregistered", TestEvent::Ping);
        // No panic or error expected
    }
}

