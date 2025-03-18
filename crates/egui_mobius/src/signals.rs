#![allow(dead_code)]
//! The Signal struct is a non-threaded mspc channel sender.
//! 
//! ```Signal<T>``` is used to send messages either command or response type 
//! to the ```Slot<T>``` receiver. Alternatively, the Dispatcher or AsyncDispatcher
//! can be used to dispatch messages to the ```Slot<T>``` receiver, while also 
//! registering the ```Slot<T>``` with the ```Dispatcher``` or ```AsyncDispatcher```
//! 


use std::sync::mpsc::Sender;

/// Signal struct with send and send_multiple methods.
pub struct Signal<T> {
    pub sender: Sender<T>,
}

impl<T> Signal<T>
where
    T: Send + 'static,
{
    /// Create a new Signal instance with a ```Sender<T>``` instance.
    /// 
    /// Example Usage:
    /// ```rust
    /// use egui_mobius::factory::create_signal_slot; 
    /// use egui_mobius::signals::Signal;
    /// 
    /// let (signal, _slot) = create_signal_slot::<String>();
    /// signal.send("Hello".to_string());
    /// ```
    pub fn new(sender: Sender<T>) -> Self {
        Signal { sender }
    }

    /// Send a ```message<T>``` to the ```Signal<T>``` instance. Typically, 
    /// the ```message<T>```  is an Event, Command, or Response type
    /// but can be any type that implements the Send trait.
    pub fn send(&self, cmd_or_msg: T) -> Result<(), String> {
        if let Err(e) = self.sender.send(cmd_or_msg) {
            eprintln!("\n***** Failed to send command: {:?}", e);
            return Err(format!("Failed to send command: {:?}", e));
        }
        Ok(())
    }
    /// Send multiple `messages<T>` to the `Signal<T>` instance. This is
    /// a convenience function that allows one to send multiple messages
    /// to the `Signal<T>` instance in a single call.
    pub fn send_multiple(&self, cmd_or_msg_vec: Vec<T>) -> Result<(), String> {
        for cmd_or_msg in cmd_or_msg_vec {
            if let Err(e) = self.sender.send(cmd_or_msg) {
                eprintln!("\n***** Failed to send command: {:?}", e);
                return Err(format!("Failed to send command: {:?}", e));
            }
        }
        Ok(())
    }
}


/// ```Clone``` trait implementation for ```Signal<T>```
/// 
/// This is important not to use #[derive(Clone)] because the ```Sender<T>``` is not
/// ```Clone``` and the ```Sender<T>``` is the only field in the ```Signal<T>``` struct.
/// 
/// Example Usage:
/// ```rust
/// use egui_mobius::signals::Signal;
/// use egui_mobius::factory::create_signal_slot;
/// 
/// let (signal, _) = create_signal_slot::<String>();
/// let cloned_signal = signal.clone();
/// ```
impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        Signal {
            sender: self.sender.clone(),
        }
    }
}
