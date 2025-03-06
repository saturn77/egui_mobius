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

    pub fn send(&self, command: T) -> Result<(), String> {
        if let Err(e) = self.sender.send(command) {
            eprintln!("\n***** Failed to send command: {:?}", e);
            return Err(format!("Failed to send command: {:?}", e));
        }
        Ok(())
    }

    pub fn send_multiple(&self, commands: Vec<T>) -> Result<(), String> {
        for command in commands {
            if let Err(e) = self.sender.send(command) {
                eprintln!("\n***** Failed to send command: {:?}", e);
                return Err(format!("Failed to send command: {:?}", e));
            }
        }
        Ok(())
    }
}