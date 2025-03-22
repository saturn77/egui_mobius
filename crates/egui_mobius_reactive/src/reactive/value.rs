use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::{channel, Sender};

use parking_lot::Mutex as PLMutex;

/// A thread-safe container for values that can be monitored for changes.
///
/// The `Value` struct allows you to store a value in a thread-safe manner and
/// provides mechanisms to monitor changes to the value.
///
/// # Example
/// ```rust
/// use egui_mobius_reactive::Value;
///
/// let value = Value::new(42);
/// assert_eq!(value.get(), 42);
///
/// value.set(84);
/// assert_eq!(value.get(), 84);
/// ```
#[derive(Clone)]
pub struct Value<T> {
    /// The inner value stored in a thread-safe `Mutex`.
    pub(crate) inner: Arc<Mutex<T>>,
    /// A list of notifiers (channels) to notify listeners when the value changes.
    notifiers: Arc<PLMutex<Vec<Sender<()>>>>,
}

impl<T> Value<T> {
    /// Gets a lock on the inner value.
    ///
    /// This method provides direct access to the inner value by locking the `Mutex`.
    ///
    /// # Returns
    /// A `MutexGuard` to the inner value.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::Value;
    ///
    /// let value = Value::new(42);
    /// let mut guard = value.lock();
    /// *guard = 84;
    /// assert_eq!(*guard, 84);
    /// ```
    pub fn lock(&self) -> std::sync::MutexGuard<T> {
        self.inner.lock().unwrap()
    }
}

impl<T: Clone + Send + 'static> Value<T> {
    /// Creates a new `Value` with the given initial value.
    ///
    /// # Arguments
    /// * `initial` - The initial value to store in the `Value`.
    ///
    /// # Returns
    /// A new `Value` instance.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::Value;
    ///
    /// let value = Value::new(42);
    /// assert_eq!(value.get(), 42);
    /// ```
    pub fn new(initial: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(initial)),
            notifiers: Arc::new(PLMutex::new(Vec::new())),
        }
    }

    /// Gets the current value.
    ///
    /// # Returns
    /// A clone of the current value.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::Value;
    ///
    /// let value = Value::new(42);
    /// assert_eq!(value.get(), 42);
    /// ```
    pub fn get(&self) -> T {
        self.inner.lock().unwrap().clone()
    }

    /// Sets a new value.
    ///
    /// This method updates the stored value and notifies all registered listeners.
    ///
    /// # Arguments
    /// * `value` - The new value to set.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::Value;
    ///
    /// let value = Value::new(42);
    /// value.set(84);
    /// assert_eq!(value.get(), 84);
    /// ```
    pub fn set(&self, value: T) {
        let mut guard = self.inner.lock().unwrap();
        *guard = value;
        drop(guard); // Release lock before notifications

        // Notify all listeners
        let notifiers = self.notifiers.lock();
        for notifier in notifiers.iter() {
            let _ = notifier.send(()); // Ignore errors from closed channels
        }
        drop(notifiers); // Explicitly release lock
    }
}

impl<T: PartialEq> PartialEq for Value<T> {
    /// Compares two `Value` instances for equality.
    ///
    /// # Arguments
    /// * `other` - The other `Value` instance to compare with.
    ///
    /// # Returns
    /// `true` if the inner values are equal, `false` otherwise.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::Value;
    ///
    /// let value1 = Value::new(42);
    /// let value2 = Value::new(42);
    /// assert_eq!(value1, value2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        *self.lock() == *other.lock()
    }
}

/// Extension trait for monitoring value changes.
///
/// This trait provides a mechanism to register callbacks that are invoked
/// whenever the value changes.
///
/// # Example
/// ```rust
/// use egui_mobius_reactive::reactive::{Value, ValueExt}; // Correct import
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use std::sync::Arc;
///
/// let value = Value::new(0);
/// let changed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
/// let changed_clone = changed.clone();
///
/// value.on_change(move || {
///     changed_clone.store(true, std::sync::atomic::Ordering::SeqCst);
/// });
///
/// value.set(42);
/// assert!(changed.load(std::sync::atomic::Ordering::SeqCst));
/// ```
pub trait ValueExt<T: Clone + Send + Sync + 'static> {
    /// Registers a callback to be called when the value changes.
    ///
    /// The callback is invoked in a dedicated background thread that waits for change notifications.
    ///
    /// # Arguments
    /// * `callback` - The callback function to invoke when the value changes.
    ///
    /// # Returns
    /// An `Arc` to the callback function.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::{Value, ValueExt};
    ///
    /// let value = Value::new(0);
    /// value.on_change(move || {
    ///     println!("Value changed!");
    /// });
    /// value.set(42);
    /// ```
    fn on_change<F>(&self, callback: F) -> Arc<F>
    where
        F: Fn() + Send + Sync + 'static;
}

impl<T: Clone + Send + Sync + PartialEq + 'static> ValueExt<T> for Value<T> {
    fn on_change<F>(&self, callback: F) -> Arc<F>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let cb = Arc::new(callback);
        let cb_clone = cb.clone();
        
        // Create a channel for change notifications
        let (tx, rx) = channel();
        
        // Add the sender to our notifiers
        self.notifiers.lock().push(tx);
        
        // Spawn a background thread to wait for notifications
        thread::spawn(move || {
            while rx.recv().is_ok() {
                cb_clone();
            }
        });

        cb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use crate::reactive::ValueExt; // Import the ValueExt trait

    /// Tests the `get` and `set` methods of the `Value` struct.
    #[test]
    fn test_value_get_set() {
        let value = Value::new(42);
        assert_eq!(value.get(), 42);

        value.set(84);
        assert_eq!(value.get(), 84);
    }

    /// Tests the `on_change` method of the `Value` struct.
    #[test]
    fn test_value_change_notification() {
        let value = Value::new(0);
        let changed = Arc::new(AtomicBool::new(false));
        let changed_clone = changed.clone();

        value.on_change(move || {
            changed_clone.store(true, Ordering::SeqCst);
        });

        // Initial state
        assert!(!changed.load(Ordering::SeqCst));

        // Change the value
        value.set(42);

        // Wait for thread startup and change detection
        thread::sleep(Duration::from_millis(50));
        assert!(changed.load(Ordering::SeqCst));
    }
}
