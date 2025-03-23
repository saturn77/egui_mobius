use std::sync::{Arc, Mutex};
use crate::reactive::Dynamic;
use crate::reactive::core::ReactiveValue;

/// Type alias for a list of subscribers.
///
/// This is used to store callbacks that should be executed when the derived value changes.
/// 
/// # Example
/// ```rust
/// use egui_mobius_reactive::reactive::Derived;
/// use egui_mobius_reactive::ReactiveValue;
/// use std::sync::Arc;
///
/// let count = Arc::new(Derived::new(&[], || 0));
/// count.subscribe(Box::new(|| println!("Value changed!"))); // Add a subscriber
/// ```
type Subscribers = Arc<Mutex<Vec<Box<dyn Fn() + Send + Sync>>>>;

/// A computed value that automatically updates when its dependencies change.
///
/// # Example
/// ```rust
/// use egui_mobius_reactive::{Dynamic, Derived};
/// use std::sync::Arc;
/// use std::thread;
/// use std::time::Duration;
///
/// let count = Dynamic::new(0);
/// let count_arc = Arc::new(count.clone());
/// let doubled = Derived::new(&[count_arc.clone()], move || {
///     let val = *count_arc.lock();
///     val * 2
/// });
/// count.set(5);  // Update the source value
/// thread::sleep(Duration::from_millis(50));
/// assert_eq!(doubled.get(), 10);  // Derived value updates automatically
/// ```
#[derive(Clone)]
pub struct Derived<T: Clone + Send + Sync + 'static> {
    /// The current value of the derived signal, stored in a thread-safe `Mutex`.
    value: Arc<Mutex<T>>,
    /// List of subscribers to notify when the value changes.
    subscribers: Subscribers,
}

/// Implementation of the `Derived` struct.
/// 
/// # Example
/// ```rust
/// use egui_mobius_reactive::{Dynamic, Derived};
/// use std::sync::Arc;
/// use std::thread;
/// use std::time::Duration;
///
/// let count = Dynamic::new(0);
/// let count_arc = Arc::new(count.clone());
/// let doubled = Derived::new(&[count_arc.clone()], move || {
///     let val = *count_arc.lock();
///     val * 2
/// });
/// count.set(5);  // Update the source value
/// thread::sleep(Duration::from_millis(50));
/// assert_eq!(doubled.get(), 10);  // Derived value updates automatically
/// ```
impl<T: Clone + Send + Sync + 'static> Derived<T> {
    /// Creates a new derived value that depends on the given reactive sources.
    pub fn new<F>(deps: &[Arc<dyn ReactiveValue>], compute: F) -> Self
    where
        F: Fn() -> T + Send + Sync + Clone + 'static,
    {
        let initial = compute();
        let value = Arc::new(Mutex::new(initial));
        let subscribers: Subscribers = Arc::new(Mutex::new(Vec::new()));

        let compute = Arc::new(compute);
        let value_clone = value.clone();
        let subs_clone = subscribers.clone();

        for dep in deps {
            let compute = compute.clone();
            let value = value.clone();
            let subs = subscribers.clone();
            dep.subscribe(Box::new(move || {
                let new_value = compute();
                *value.lock().unwrap() = new_value;
                for cb in subs.lock().unwrap().iter() {
                    cb();
                }
            }));
        }

        Self {
            value: value_clone,
            subscribers: subs_clone,
        }
    }

    /// Gets the current value of the derived signal.
    pub fn get(&self) -> T {
        self.value.lock().unwrap().clone()
    }

    /// Registers a callback to be called whenever the derived value changes.
    pub fn on_change(&self, f: Box<dyn Fn() + Send + Sync>) {
        self.subscribers.lock().unwrap().push(f);
    }
}

impl<T: Clone + Send + Sync + 'static> From<Derived<T>> for Dynamic<T> {
    fn from(val: Derived<T>) -> Self {
        let initial_value = val.get();
        Dynamic::new(initial_value)
    }
}

impl<T: Clone + Send + Sync + 'static> ReactiveValue for Derived<T> {
    fn subscribe(&self, f: Box<dyn Fn() + Send + Sync>) {
        self.on_change(f);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_derived_updates() {
        let count = Dynamic::new(0);
        let count_for_compute = count.clone();
        let doubled = Derived::new(&[Arc::new(count.clone())], move || {
            *count_for_compute.lock() * 2
        });

        assert_eq!(doubled.get(), 0);

        count.set(5);
        thread::sleep(Duration::from_millis(50));
        assert_eq!(doubled.get(), 10);
    }

    #[test]
    fn test_derived_multiple_deps() {
        let a = Dynamic::new(1);
        let b = Dynamic::new(2);
        let a_for_compute = a.clone();
        let b_for_compute = b.clone();
        let sum = Derived::new(&[Arc::new(a.clone()), Arc::new(b.clone())], move || {
            *a_for_compute.lock() + *b_for_compute.lock()
        });

        assert_eq!(sum.get(), 3);

        a.set(5);
        thread::sleep(Duration::from_millis(50));
        assert_eq!(sum.get(), 7);

        b.set(3);
        thread::sleep(Duration::from_millis(50));
        assert_eq!(sum.get(), 8);
    }
}
