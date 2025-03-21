//! The types module provides core types for implementing the design patterns and
//! architecture of egui_mobius.
//! 
//! This module contains three primary types:
//! - **Value**: A heap-allocated, thread-safe container for shared state
//! - **ValueGuard**: A RAII guard for safe access to Value contents
//! - **Edge**: A utility for detecting state transitions in signals
//!
//! The **Value** type wraps `Arc<Mutex<T>>` to provide a cleaner interface for
//! thread-safe state sharing, commonly used with Signal and Slot types. It
//! eliminates the need to directly handle atomic reference counting and mutex
//! operations in client code.
//! 
//! The **ValueGuard** type implements the RAII pattern to ensure safe access
//! to Value contents, automatically handling mutex locking and unlocking.
//!
//! The **Edge** type provides utilities for detecting rising and falling edges
//! in signal transitions, particularly useful in immediate mode GUI contexts.
//! 


use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

pub type Enqueue<T> = std::sync::mpsc::Sender<T>;
pub type Dequeue<T> = std::sync::mpsc::Receiver<T>;

pub type EventEnqueue<T> = tokio::sync::mpsc::Sender<T>;
pub type EventDequeue<T> = tokio::sync::mpsc::Receiver<T>;

/// The Value Type - heap allocated and thread safe.
/// 
/// The Value type is heap allocated and thread safe type that can be used to store
/// a value that can be shared across multiple threads, useful for shared state or for
/// Signal and Slot types.  
///
/// The value can be read and written using the `read` and `write` methods. The value 
/// can be locked using the `lock` method which returns a `ValueGuard` that can be used 
/// to read and write the value.
/// 
/// Example Usage:
/// ```rust
/// use egui_mobius::types::Value;
/// use egui_mobius::signals::Signal;
/// use std::fmt::Debug;
/// 
/// #[derive(Debug, Clone)]
/// pub enum Event {
///     IncrementCounter,
///     ResetCounter,
///     Custom(String), 
/// }
/// 
/// #[derive(Default, Clone)]
/// pub struct DashboardState {
///     pub counter: usize,
/// }
/// #[derive(Debug, Clone)]
///pub enum Response {
///    CounterUpdated(usize),
///    Message(String), // For general-purpose backend messages
///}
///
/// impl DashboardState {
///     pub fn handle_response(&mut self, response: Response) {
///         if let Response::CounterUpdated(value) = response {
///             self.counter = value;
///         }
///     }
/// }
///
/// #[derive(Clone)]
/// pub struct AppState {
///     pub dashboard: DashboardState,
///     pub event_signal: Signal<Event>,
///     pub logs: Vec<String>,
///     pub log_filter: String,
/// }
/// 
/// pub struct UiApp {
///     state        : Value<AppState>,
///     event_signal : Signal<Event>,
/// }
/// 
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Value<T>(Arc<Mutex<T>>);

impl<T: Default> Default for Value<T> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(T::default())))
    }
}

impl<T: Debug> Debug for Value<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Value")
            .field(&self.0)
            .finish()
    }
}

impl<T> Clone for Value<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Value<T> {
    // TODO avoid exposing `PoisonError` in the API here.
    pub fn lock(&self) -> Result<ValueGuard<'_, T>, PoisonError<MutexGuard<'_, T>>> {
        let result = self.0.lock().map(|result| ValueGuard(result));
        result
    }

    /// Create a new Value instance with the given value of type T.
    pub fn new(value: T) -> Value<T> {
        Self(Arc::new(Mutex::new(value)))
    }

    /// Write a value of type T to the Value instance.
    pub fn write (&self, value: T) {
        let mut guard = self.lock().unwrap();
        *guard = value;
    }

    /// Read a value of type T from the Value instance.
    pub fn read(&self) -> T
    where
        T: Clone,
    {
        let guard = self.lock().unwrap();
        guard.clone()
    }

    /// Make aliases of get (read) and set (write) for additional ergonomics
    pub fn get(&self) -> T 
    where T: Clone 
    {
        self.read()
    }

    /// Make aliases of get (read) and set (write) for additional ergonomics
    pub fn set(&self, value: T) {
        self.write(value);
    }

}

impl<T: Send> Value<T> {}

/// ValueGuard type - Mutex Guard for the Value type.
///
/// The ValueGuard type is a guard type that is used to lock the `Value` type and
/// provides a way to deref the value either mutably or immutably.
pub struct ValueGuard<'a, T>(MutexGuard<'a, T>);

impl<T> Deref for ValueGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl<T> DerefMut for ValueGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.deref_mut()
    }
}

// need to implement push_back for VecDeque
// This will facilitate the producer thread to send messages to the UI
// in an ergonomic way.
use std::collections::VecDeque;
impl<T> Value<VecDeque<T>> {
    pub fn push_back(&self, value: T) {
        let mut guard = self.lock().unwrap();
        guard.push_back(value);
    }
}

///
/// Edge Type
///
/// This type is used to detect edges in the input signal.
/// It is used to detect rising and falling edges in the input signal.
/// The type T should implement the following traits:
/// - Clone
/// - Debug
/// - Display
/// - PartialEq
/// - PartialOrd
/// - Send
/// - 'static
///   The type T is the type of the input signal.
/// 
/// The Edge type is used to detect edges in the input signal, particularly
/// since egui is an immediate mode GUI library, it is important to detect
/// when the input signal changes so that the UI can be updated accordingly.
/// 
/// The goal is to reduce clutter within the App struct and to make the
/// code more readable and maintainable.
#[derive(Clone, Debug)]
pub struct Edge<T>
where
    T: Clone + Debug + Display + PartialEq + PartialOrd + Send + 'static,
{
    pub values: Vec<T>,
}

impl<T> Display for Edge<T>
where
    T: Clone + Debug + Display + PartialEq + PartialOrd + Send + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Edge(values: {:?}", self.values)
    }
}

impl PartialEq for Edge<String> {
    fn eq(&self, other: &Self) -> bool {
        self.values == other.values
    }
}


impl<T> Edge<T>
where
    T: Clone + Debug + Display + PartialEq + PartialOrd + Send + 'static,
{
    /// Create a new Edge instance with the given value of type T.
    pub fn new(value: T) -> Self {
        Self {
            values: vec![value.clone(), value],
        }
    }

    /// Add a new value to the Edge instance.
    pub fn add_value(&mut self, new_value: T) {
        self.values[1] = self.values[0].clone();
        self.values[0] = new_value;
    }

    /// Check if the values are equal.
    pub fn are_values_equal(&self) -> bool {
        self.values[0] == self.values[1]
    }

    /// Check if a positive edge is detected.
    pub fn positive_edge_detect(&self) -> bool {
        self.values[0] != self.values[1] && self.values[0] > self.values[1]
    }

    /// Check if a negative edge is detected.
    pub fn negative_edge_detect(&self) -> bool {
        self.values[0] != self.values[1] && self.values[0] < self.values[1]
    }
}


//-------------------------------------------------------------------------
// ** Tests **
//-------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    //---------------------------------------------------------------------
    // Unit tests for the Value Type
    //---------------------------------------------------------------------
    #[test]
    fn test_value() {
        let value = Value::new(0);
        assert_eq!(*value.lock().unwrap(), 0);

        *value.lock().unwrap() = 1;
        assert_eq!(*value.lock().unwrap(), 1);

        assert_eq!(value.read(), 1);
        value.write(2);
        assert_eq!(value.read(), 2);

        assert_eq!(value.get(), 2);
        value.set(3);
        assert_eq!(value.get(), 3);

        let value = Value::new("hello".to_string());
        assert_eq!(*value.lock().unwrap(), "hello".to_string());

        *value.lock().unwrap() = "world".to_string();
        assert_eq!(*value.lock().unwrap(), "world".to_string());

        assert_eq!(value.read(), "world".to_string());
        value.write("world".to_string());
        assert_eq!(value.read(), "world".to_string());

        assert_eq!(value.get(), "world".to_string());
        value.set("hello".to_string());
        assert_eq!(value.get(), "hello".to_string());

        // also test write / set for Value 
        let value = Value::new(0);
        value.write(1);
        assert_eq!(value.read(), 1);

        let value = Value::new("hello".to_string());
        value.write("world".to_string());
        assert_eq!(value.read(), "world".to_string());

        // also test the alias for get and set
        let value = Value::new(0);
        value.set(1);
        assert_eq!(value.get(), 1);

        let value = Value::new("hello".to_string());
        value.set("world".to_string());
        assert_eq!(value.get(), "world".to_string());

    }

    //---------------------------------------------------------------------
    // Unit tests for the Value Type
    //---------------------------------------------------------------------
    #[test]
    fn test_edge() {
        let mut edge = Edge::new(0);
        assert_eq!(edge.are_values_equal(), true);

        edge.add_value(1);
        assert_eq!(edge.are_values_equal(), false);
        assert_eq!(edge.positive_edge_detect(), true);
        assert_eq!(edge.negative_edge_detect(), false);

        edge.add_value(1);
        assert_eq!(edge.are_values_equal(), true);

        edge.add_value(2);
        assert_eq!(edge.are_values_equal(), false);
        assert_eq!(edge.positive_edge_detect(), true);
        assert_eq!(edge.negative_edge_detect(), false);

        edge.add_value(1);
        assert_eq!(edge.are_values_equal(), false);
        assert_eq!(edge.positive_edge_detect(), false);
        assert_eq!(edge.negative_edge_detect(), true);

        // add test case for string
        let mut edge = Edge::new("hello".to_string());
        edge.add_value("world".to_string());
        assert_eq!(edge.are_values_equal(), false);

    }
}

