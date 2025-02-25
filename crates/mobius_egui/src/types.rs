use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

pub type Enqueue<T> = std::sync::mpsc::Sender<T>;
pub type Dequeue<T> = std::sync::mpsc::Receiver<T>;

pub type EventEnqueue<T> = tokio::sync::mpsc::Sender<T>;
pub type EventDequeue<T> = tokio::sync::mpsc::Receiver<T>;

pub struct Value<T>(Arc<Mutex<T>>);

impl<T: Default> Default for Value<T> {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(T::default())))
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
        
        let result = self.0.lock()
            .map(|result|ValueGuard(result));
        
        result
    }

    pub fn new(value: T) -> Value<T> {
        Self(Arc::new(Mutex::new(value)))
    }
}

impl<T: Send> Value<T> {
}

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
