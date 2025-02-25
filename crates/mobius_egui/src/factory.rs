use std::sync::mpsc::{self, Sender, Receiver};
use crate::signals::Signal;
use crate::slot::Slot;

pub fn create_signal_slot<T>() -> (Signal<T>, Slot<T>)
where
    T: Send + 'static,
{
    let (tx, rx): (Sender<T>, Receiver<T>) = mpsc::channel();
    let signal = Signal::new(tx);
    let slot = Slot::new(rx);
    (signal, slot)
}