use std::sync::Arc;
use crate::reactive::{Value, Derived, SignalRegistry};
use crate::reactive::value::ValueExt;

/// Example showing how to use the reactive system in a real application.
/// This example demonstrates:
/// - Creating reactive values
/// - Creating computed values that depend on other values
/// - Automatic updates when values change
/// - Thread-safe state management
pub struct CounterApp {
    pub count: Value<i32>,
    pub label: Value<String>,
    pub doubled: Derived<i32>,
    registry: SignalRegistry,
}

impl CounterApp {
    pub fn new() -> Arc<Self> {
        let registry = SignalRegistry::new();
        let count = Value::new(0);
        
        // Create a derived value that automatically updates when count changes
        let doubled = Derived::new(&[count.clone()], move || {
            let val = *count.lock().unwrap();
            val * 2
        });

        // Create a label that updates when count changes
        let label = Value::new("Count is 0".to_string());
        let count_for_label = count.clone();
        count.on_change(move || {
            let val = *count_for_label.lock().unwrap();
            label.set(format!("Count is {}", val));
        });

        // Register values with the registry
        registry.register_signal(Arc::new(count.clone()));
        registry.register_signal(Arc::new(label.clone()));
        registry.register_signal(Arc::new(doubled.clone()));

        Arc::new(Self {
            count,
            label,
            doubled,
            registry,
        })
    }

    pub fn increment(&self) {
        let mut count = self.count.lock();
        *count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_counter_app() {
        let app = CounterApp::new();
        
        // Test initial state
        assert_eq!(*app.count.inner.lock().unwrap(), 0);
        assert_eq!(app.doubled.get(), 0);
        assert_eq!(app.label.get(), "Count is 0");

        // Test increment
        app.increment();
        thread::sleep(Duration::from_millis(200));

        assert_eq!(*app.count.inner.lock().unwrap(), 1);
        assert_eq!(app.doubled.get(), 2);
        assert_eq!(app.label.get(), "Count is 1");
    }
}
