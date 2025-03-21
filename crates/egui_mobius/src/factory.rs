//! Factory module for creating signal-slot pairs in egui_mobius.
//!
//! This module provides utility functions for creating type-safe communication channels
//! between components in an egui_mobius application. It implements the Factory pattern
//! to create properly configured signal-slot pairs that work with the egui_mobius
//! message passing system.
//!
//! # Example
//! ```rust
//! use egui_mobius::factory::create_signal_slot;
//!
//! // Create a signal-slot pair for integer messages
//! let (signal, mut slot) = create_signal_slot::<i32>();
//!
//! // Set up the receiving end
//! slot.start(|value| {
//!     println!("Received value: {}", value);
//! });
//!
//! // Send a value through the signal
//! signal.send(42).unwrap();
//! ```

use std::sync::mpsc::{self, Sender, Receiver};
use crate::signals::Signal;
use crate::slot::Slot;

/// Creates a new signal-slot pair.
///
/// This is a utility function that creates a new signal-slot pair for type-safe
/// message passing between components. The signal can be used to send messages,
/// while the slot can be used to receive and process them.
///
/// # Type Parameters
/// * `T` - The type of messages that will be passed through this signal-slot pair
///
/// # Returns
/// A tuple containing:
/// * `Signal<T>` - The sending end of the channel
/// * `Slot<T>` - The receiving end of the channel
///
/// # Example
/// ```rust
/// use egui_mobius::factory::create_signal_slot;
///
/// // Create a signal-slot pair for string messages
/// let (signal, mut slot) = create_signal_slot::<String>();
///
/// // Set up a handler for the slot
/// slot.start(|message| {
///     println!("Received: {}", message);
/// });
///
/// // Send a message through the signal
/// signal.send("Hello!".to_string()).unwrap();
/// ```
pub fn create_signal_slot<T>() -> (Signal<T>, Slot<T>)
where
    T: Send + Clone + 'static,
{
    let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();
    let signal = Signal::new(tx);
    let slot = Slot::new(rx);
    (signal, slot)
}
