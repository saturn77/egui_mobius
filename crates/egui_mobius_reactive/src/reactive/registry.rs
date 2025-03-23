use std::sync::{Arc, Mutex};
use crate::reactive::ReactiveValue;
use std::any::Any;

/// Alias for shared reactive signal type
pub type SharedReactive = Arc<dyn ErasedReactiveValue>;

/// Trait alias for ReactiveValue + Any
pub trait ErasedReactiveValue: ReactiveValue + Any {}
impl<T: ReactiveValue + Any> ErasedReactiveValue for T {}

/// A registry that manages reactive values and their dependencies.
///
/// The registry is used to keep track of all reactive values in the system.
/// This is useful for:
/// - Ensuring values aren't dropped while they're still needed
/// - Managing the lifecycle of reactive values
/// - Debugging and visualizing the reactive graph
#[derive(Clone, Default)]
pub struct SignalRegistry {
    signals: Arc<Mutex<Vec<(String, SharedReactive)>>>,
}

impl SignalRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            signals: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a named signal.
    pub fn register_named_signal(&self, name: &str, signal: SharedReactive) {
        self.signals.lock().unwrap().push((name.to_string(), signal));
    }

    /// List all registered signals and their names.
    pub fn list_signals(&self) -> Vec<(String, SharedReactive)> {
        self.signals.lock().unwrap().clone()
    }

    /// Attach an effect that runs whenever any of the given dependencies change.
    ///
    /// # Notes on `'static` bound for dependencies:
    /// - `'static` **does not mean the value lives forever**
    /// - It means the type must **own all its data**, not borrow it
    /// - Ensures compatibility with threads and long-term storage
    /// - Required for safely storing trait objects like `Arc<dyn ReactiveValue>`
    /// - All common reactive types (`Value<T>`, `Derived<T>`, `ReactiveList<T>`) satisfy `'static`
    pub fn effect<F>(&self, deps: &[SharedReactive], f: F)
    where
        F: Fn() + 'static + Send + Sync,
    {
        let f = Arc::new(Mutex::new(f));

        for dep in deps {
            let f = f.clone();
            dep.subscribe(Box::new(move || {
                if let Ok(f) = f.lock() {
                    f();
                }
            }));
        }

        // Run once initially
        if let Ok(f) = f.lock() {
            f();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reactive::{Value, Derived};
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_registry_keeps_signals_alive() {
        let registry = SignalRegistry::new();

        let count = Value::new(0);
        let count_for_compute = count.clone();
        let doubled = Derived::new(&[Arc::new(count.clone())], move || {
            *count_for_compute.lock() * 2
        });

        // Register the values
        registry.register_named_signal("count", Arc::new(count.clone()));
        registry.register_named_signal("doubled", Arc::new(doubled.clone()));

        // Values should still work after registration
        assert_eq!(doubled.get(), 0);
        count.set(5);
        thread::sleep(Duration::from_millis(50));
        assert_eq!(doubled.get(), 10);
    }
}
