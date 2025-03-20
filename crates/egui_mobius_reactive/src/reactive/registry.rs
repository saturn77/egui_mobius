use std::sync::{Arc, Mutex};
use std::any::Any;

/// A registry that manages reactive values and their dependencies.
///
/// The registry is used to keep track of all reactive values in the system.
/// This is useful for:
/// - Ensuring values aren't dropped while they're still needed
/// - Managing the lifecycle of reactive values
/// - Future extensions (e.g., debugging, visualization of dependency graphs)
#[derive(Clone, Default)]
pub struct SignalRegistry {
    signals: Arc<Mutex<Vec<Arc<dyn Any + Send + Sync>>>>,
}

impl SignalRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            signals: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Registers a signal with the registry.
    ///
    /// This ensures the signal isn't dropped while it's still needed.
    pub fn register_signal<T: 'static + Send + Sync>(&self, signal: Arc<T>) {
        self.signals.lock().unwrap().push(signal);
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
        let doubled = Derived::new(&[count.clone()], move || {
            *count_for_compute.lock() * 2
        });

        // Register the values
        registry.register_signal(Arc::new(count.clone()));
        registry.register_signal(Arc::new(doubled.clone()));

        // Values should still work after registration
        assert_eq!(doubled.get(), 0);
        count.set(5);
        thread::sleep(Duration::from_millis(50));
        assert_eq!(doubled.get(), 10);
    }
}
