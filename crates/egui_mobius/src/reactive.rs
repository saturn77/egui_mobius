use std::sync::{Arc, Mutex};
use std::any::Any;
use crate::types::Value;
use crate::slot::Slot;

// === Value Extension === //
pub trait ValueExt<T: Clone + Send + Sync + 'static> {
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
        let value = self.clone();
        
        // Create a thread to monitor changes
        std::thread::spawn(move || {
            let mut last_value = None;
            loop {
                let current = value.lock().unwrap().clone();
                if last_value.as_ref() != Some(&current) {
                    last_value = Some(current);
                    cb_clone();
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        });
        
        cb
    }
}

// === Signal Value === //
type Callback = Box<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub struct SignalValue<T: Clone> {
    value: Arc<Mutex<T>>,
    subscribers: Arc<Mutex<Vec<Callback>>>,
}

impl<T: Clone + std::fmt::Debug> std::fmt::Debug for SignalValue<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SignalValue")
            .field("value", &self.value)
            .field("subscribers_count", &self.subscribers.lock().unwrap().len())
            .finish()
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> SignalValue<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(Mutex::new(initial)),
            subscribers: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn get(&self) -> T {
        self.value.lock().unwrap().clone()
    }

    pub fn set(&self, new: T) {
        let mut value = self.value.lock().unwrap();
        if *value != new {
            *value = new;
            self.notify();
        }
    }

    pub fn subscribe(&self, cb: Callback) {
        self.subscribers.lock().unwrap().push(cb);
    }

    fn notify(&self) {
        for cb in self.subscribers.lock().unwrap().iter() {
            cb();
        }
    }
}

// === Derived Signal === //
pub struct Derived<T: Clone> {
    value: Arc<Mutex<T>>,
    compute: Arc<dyn Fn() -> T + Send + Sync>,
    _subscriptions: Vec<Box<dyn Any + Send + Sync>>, // Keep subscriptions alive
}

impl<T: Clone + 'static + Send> Derived<T> {
    pub fn new<F, D>(deps: &[Value<D>], compute: F) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
        D: Clone + Send + Sync + PartialEq + 'static,
    {
        let compute = Arc::new(compute);
        let initial = compute();
        let value = Arc::new(Mutex::new(initial));
        
        // Create subscriptions to all dependencies
        let mut subscriptions = Vec::new();
        for dep in deps {
            let value = value.clone();
            let compute = compute.clone();
            let sub = dep.on_change(move || {
                let new_val = compute();
                *value.lock().unwrap() = new_val;
            });
            subscriptions.push(Box::new(sub) as Box<dyn Any + Send + Sync>);
        }

        Self {
            value,
            compute,
            _subscriptions: subscriptions,
        }
    }

    pub fn get(&self) -> T {
        let mut value = self.value.lock().unwrap();
        // Recompute value using compute function
        *value = (self.compute)();
        value.clone()
    }
}

// === Signal Registry === //
#[derive(Clone)]
pub struct SignalRegistry {
    signals: Arc<Mutex<Vec<Arc<dyn Any + Send + Sync>>>>,
}

impl SignalRegistry {
    pub fn new() -> Self {
        Self {
            signals: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register_signal<T: 'static + Send + Sync>(&self, signal: Arc<T>) {
        self.signals.lock().unwrap().push(signal);
    }

    pub fn register_signal_handler<T, F>(&self, mut slot: Slot<T>, handler: F)
    where
        T: Clone + Send + 'static,
        F: Fn(T) + Send + Sync + 'static,
    {
        slot.start(handler);
    }
}

// === Declarative Macro === //
#[macro_export]
macro_rules! signal {
    ($name:ident : $ty:ty = $val:expr) => {
        let $name = SignalValue::<$ty>::new($val);
    };
}
