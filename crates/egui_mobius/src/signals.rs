#![allow(dead_code)]

// improved concept of the Signal! macro
// Signal struct with send and send_multiple methods
// Signal struct is used to send commands to the MobiusEnque<Command> sender

use std::sync::mpsc::Sender;

#[derive(Clone)]
pub struct Signal<T> {
    pub sender: Sender<T>,
}

impl<T> Signal<T>
where
    T: Send + 'static,
{
    pub fn new(sender: Sender<T>) -> Self {
        Signal { sender }
    }

    pub fn send(&self, cmd_or_msg: T) -> Result<(), String> {
        if let Err(e) = self.sender.send(cmd_or_msg) {
            eprintln!("\n***** Failed to send command: {:?}", e);
            return Err(format!("Failed to send command: {:?}", e));
        }
        Ok(())
    }

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