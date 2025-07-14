//! Dynamic<T> is a thread-safe container for dynamic values that can be monitored for changes.
//! 
//! The `Dynamic` struct allows you to store a value in a thread-safe manner and
//! provides mechanisms to monitor changes to the value. It is often on the argument list to the 
//! UiState or AppState function.  
//! 
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::fmt::{self, Debug};
use parking_lot::Mutex as PLMutex;
use crate::ReactiveValue;

/// A thread-safe container for dynamic values that can be monitored for changes.
///
/// The `Dynamic` struct allows you to store a value in a thread-safe manner and
/// provides mechanisms to monitor changes to the value.
///
/// # Example
/// ```rust
/// use egui_mobius_reactive::Dynamic;
///
/// let value = Dynamic::new(42);
/// assert_eq!(value.get(), 42);
///
/// value.set(84);
/// assert_eq!(value.get(), 84);
/// ```
#[derive(Clone)]
pub struct Dynamic<T> {
    /// The inner value stored in a thread-safe `Mutex`.
    pub(crate) inner: Arc<Mutex<T>>,
    /// A list of notifiers (channels) to notify listeners when the value changes.
    notifiers: Arc<PLMutex<Vec<Sender<()>>>>,
}

impl<T> Dynamic<T> {
    /// Gets a lock on the inner value.
    ///
    /// This method provides direct access to the inner value by locking the `Mutex`.
    ///
    /// # Returns
    /// A `MutexGuard` to the inner value.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::Dynamic;
    ///
    /// let value = Dynamic::new(42);
    /// let mut guard = value.lock();
    /// *guard = 84;
    /// assert_eq!(*guard, 84);
    /// ```
    pub fn lock(&self) -> std::sync::MutexGuard<'_, T> {
        self.inner.lock().unwrap()
    }
}

impl<T: Clone + Send + 'static> Dynamic<T> {
    /// Creates a new `Dynamic` with the given initial value.
    ///
    /// # Arguments
    /// * `initial` - The initial value to store in the `Dynamic`.
    ///
    /// # Returns
    /// A new `Dynamic` instance.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::Dynamic;
    ///
    /// let value = Dynamic::new(42);
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
    /// use egui_mobius_reactive::Dynamic;
    ///
    /// let value = Dynamic::new(42);
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
    /// use egui_mobius_reactive::Dynamic;
    ///
    /// let value = Dynamic::new(42);
    /// value.set(84);
    /// assert_eq!(value.get(), 84);
    /// ```
    pub fn set(&self, value: T) {
        let mut guard = self.inner.lock().unwrap();
        *guard = value;

        // Notify all listeners
        for notifier in self.notifiers.lock().iter() {
            let _ = notifier.send(()); // Ignore errors from closed channels
        }
    }
}

impl<T: PartialEq> PartialEq for Dynamic<T> {
    /// Compares two `Value` instances for equality.
    ///
    /// # Arguments
    /// * `other` - The other `Dynamic` instance to compare with.
    ///
    /// # Returns
    /// `true` if the inner values are equal, `false` otherwise.
    ///
    /// # Example
    /// ```rust
    /// use egui_mobius_reactive::Dynamic;
    ///
    /// let value1 = Dynamic::new(42);
    /// let value2 = Dynamic::new(42);
    /// assert_eq!(value1, value2);
    /// ```
    fn eq(&self, other: &Self) -> bool {
        *self.lock() == *other.lock()
    }
}

/// Implements the `Debug` trait for `Dynamic<T>` where `T` implements `Debug`.
impl<T: Debug + Clone + Send + 'static> Debug for Dynamic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Dynamic({:?})", self.get())
    }
}

/// Extension trait for monitoring value changes.
///
/// This trait provides a mechanism to register callbacks that are invoked
/// whenever the value changes.
///
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
    /// use egui_mobius_reactive::{Dynamic, ValueExt};
    ///
    /// let value = Dynamic::new(0);
    /// value.on_change(move || {
    ///     println!("Value changed!");
    /// });
    /// value.set(42);
    /// ```
    fn on_change<F>(&self, callback: F) -> Arc<F>
    where
        F: Fn() + Send + Sync + 'static;
}

impl<T: Clone + Send + Sync + PartialEq + 'static> ValueExt<T> for Dynamic<T> {
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



impl<T: Clone + Send + Sync + PartialEq + 'static> ReactiveValue for Dynamic<T> {
    fn subscribe(&self, f: Box<dyn Fn() + Send + Sync>) {
        // Directly pass the function `f` instead of wrapping it in a closure
        self.on_change(f);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Converts a `Dynamic<T>` to a `Dynamic<U>` where `T` can be converted to `U`.
/// 
/// This is useful for converting between different types of dynamic values.
/// /// # Example
/// /// ```rust
/// use egui_mobius_reactive::{Dynamic, ValueExt};
///
/// let value = Dynamic::new(42);
/// let float_value: Dynamic<f64> = value.into();
/// assert_eq!(float_value.get(), 42.0);
/// ```
impl<T> From<T> for Dynamic<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn from(value: T) -> Self {
        Dynamic::new(value)
    }
}

/// Converts a `Dynamic<T>` to a `Dynamic<f64>` where `T` can be converted to `f64`.
///
/// This is useful for converting between different types of dynamic values.
/// Note here the 'a lifetime is used to ensure that the conversion is valid for the lifetime of the `Dynamic<T>`.
/// This is important for ensuring that the conversion does not outlive the original `Dynamic<T>`.
/// This is particularly useful in a multi-threaded context where the `Dynamic<T>` may be shared across threads.
/// 
/// # Example
/// ```rust
/// use egui_mobius_reactive::{Dynamic, ValueExt};
///
/// let value = Dynamic::new(42);
/// let float_value: f64 = (&value).into();
/// assert_eq!(float_value, 42.0);
/// ```
impl<'a, T> From<&'a Dynamic<T>> for f64
where
    T: Clone + Send + Sync + Into<f64> + 'static,
{
    fn from(dynamic: &'a Dynamic<T>) -> Self {
        let value = dynamic.get();
        value.into()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::time::Duration;
    use crate::ValueExt; // Import the ValueExt trait

    /// Tests the `get` and `set` methods of the `Dynamic` struct.
    #[test]
    fn test_value_get_set() {
        let value = Dynamic::new(42);
        assert_eq!(value.get(), 42);

        value.set(84);
        assert_eq!(value.get(), 84);
    }

    /// Tests the `on_change` method of the `Dynamic` struct.
    #[test]
    fn test_value_change_notification() {
        let value = Dynamic::new(0);
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

    /// Tests the ReactiveValue trait implementation for Dynamic.
    #[test]
    fn test_reactive_value_trait() {
        let value = Dynamic::new(0);
        let changed = Arc::new(AtomicBool::new(false));
        let changed_clone = changed.clone();

        // Subscribe to value changes
        value.subscribe(Box::new(move || {
            changed_clone.store(true, Ordering::SeqCst);
        }));

        // Initial state
        assert!(!changed.load(Ordering::SeqCst));

        // Change the value
        value.set(42);

        // Wait for thread startup and change detection
        thread::sleep(Duration::from_millis(50));
        assert!(changed.load(Ordering::SeqCst));
    }

}
