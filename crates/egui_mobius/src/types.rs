use std::fmt::{self, Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

pub type Enqueue<T> = std::sync::mpsc::Sender<T>;
pub type Dequeue<T> = std::sync::mpsc::Receiver<T>;

pub type EventEnqueue<T> = tokio::sync::mpsc::Sender<T>;
pub type EventDequeue<T> = tokio::sync::mpsc::Receiver<T>;

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
    pub fn lock(&self) -> Result<ValueGuard<T>, PoisonError<MutexGuard<T>>> {
        let result = self.0.lock().map(|result| ValueGuard(result));
        result
    }

    pub fn new(value: T) -> Value<T> {
        Self(Arc::new(Mutex::new(value)))
    }
}

impl<T: Send> Value<T> {}

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

//-------------------------------------------------------------------------
// ** Edge Type **
//-------------------------------------------------------------------------
// This type is used to detect edges in the input signal.
// It is used to detect rising and falling edges in the input signal.
// The type T should implement the following traits:
// - Clone
// - Debug
// - Display
// - PartialEq
// - PartialOrd
// - Send
// - 'static
// The type T is the type of the input signal.
// The Edge type is used to detect edges in the input signal, particularly
// since egui is an immediate mode GUI library, it is important to detect
// when the input signal changes so that the UI can be updated accordingly.
// The goal is to reduce clutter within the App struct and to make the
// code more readable and maintainable.
//-------------------------------------------------------------------------
// Unit Tests : provided 
//-------------------------------------------------------------------------
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
    pub fn new(value: T) -> Self {
        Self {
            values: vec![value.clone(), value],
        }
    }

    pub fn add_value(&mut self, new_value: T) {
        self.values[1] = self.values[0].clone();
        self.values[0] = new_value;
    }

    pub fn are_values_equal(&self) -> bool {
        self.values[0] == self.values[1]
    }

    pub fn positive_edge_detect(&self) -> bool {
        self.values[0] != self.values[1] && self.values[0] > self.values[1]
    }

    pub fn negative_edge_detect(&self) -> bool {
        self.values[0] != self.values[1] && self.values[0] < self.values[1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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