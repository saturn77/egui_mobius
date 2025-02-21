use std::sync::{Arc, Mutex};


#[allow(dead_code)]
pub struct Slot<R: Clone + Send + 'static> {
    result_receiver: Arc<Mutex<Option<R>>>,
}

#[allow(dead_code)]
impl<R: Clone + Send + 'static> Slot<R> {
    pub fn new() -> Self {
        Self {
            result_receiver: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_result(&self) -> Option<R> {
        self.result_receiver.lock().unwrap().clone()
    }

    pub fn set_result(&self, result: R) {
        *self.result_receiver.lock().unwrap() = Some(result);
    }
}