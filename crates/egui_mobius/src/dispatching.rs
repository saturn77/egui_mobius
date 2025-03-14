//! Dispatcher module for egui_mobius
//!
//! This provides a generic signal-slot system using `Dispatcher` and `SignalDispatcher`.
//! It supports named channels and integrates with the `Value<T>` type from `egui_mobius`.
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
//! fn main() {
//!     let dispatcher = Dispatcher::<Event>::new();
//!
//!     dispatcher.register_slot("greet", |event| {
//!         if let Event::Text(text) = event {
//!             println!("Received: {}", text);
//!         }
//!     });
//!
//!     dispatcher.send("greet", Event::Text("hi from egui_mobius".into()));
//! }
//! ```

use std::collections::HashMap;
use crate::types::Value;

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
/// fn main() {
///     let dispatcher = Dispatcher::<MyEvent>::new();
/// }
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
    handlers: Value<HashMap<String, Vec<std::sync::Arc<dyn Fn(E) + Send + Sync>>>>,
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
    /// let dispatcher = Dispatcher::<MyEvent>::new();
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

// src/main.rs
//mod dispatching;
// mod app;
