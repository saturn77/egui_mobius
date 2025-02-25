# egui_mobius  
Modular and ergonomic construction of egui applications. 

![egui_mobius Logo](./assets/mobius_strip.png)  

**egui_mobius** is a Rust framework designed to modularize and separate front-end and back-end logic in Egui applications. It emphasizes clean architecture with seamless communication between UI and business logic, inspired by the continuous, one-sided nature of the Möbius strip.  

## Why egui_mobius?  
In traditional Egui applications, UI and backend logic are often tightly coupled. egui_mobius solves this by providing a structured approach to communication between layers, improving maintainability and scalability.  

## Features  
- Clear separation of UI and business logic.  
- Flexible command and event processing using `std::mpsc`.  
- Modular design for cleaner, more maintainable code.  

## Quick Start
There are multiple crates in the repository, and each example
is a crate. 
One can test out an example by running these steps, in this
case the simple_monitor, and just substitute an appropriate 
example name.  
```bash
git clone git@github.com:saturn77/egui_mobius.git 
cd egui_mobius
cargo build
cd examples
cargo run -p simple_monitor
```

## Installation
When using egui_mobius as a library, add the following to your `Cargo.toml`:  
```toml
[dependencies]
egui_mobius = "0.1.0"
egui = "0.31.0"
eframe = { version = "0.31.0", default-features = false, features = [
    "default_fonts", 
    "glow",          
    "wayland",       
] }
```  

## Usage  
Example of sending a command from the UI using the `Signal` struct:
```rust
if ui.button("First Task").clicked() {
    let signal = Signal::new(self.command_sender.clone());
    if let Err(e) = signal.send(Command::FirstTask) {
        eprintln!("Error sending command: {}", e);
    }
}
```

Example of setting up a `Slot` to handle commands:
```rust
use std::sync::mpsc::{self, Receiver, Sender};

fn main() {
    let (command_sender, command_receiver): (Sender<Command>, Receiver<Command>) = mpsc::channel();
    let signal = Signal::new(command_sender);
    let slot = Slot::new(command_receiver);

    // Define a handler function for the slot
    let handler = |command: Command| {
        match command {
            Command::FirstTask => {
                println!("Processing FirstTask...");
                // Process the command and send the result
                result_sender.send(CommandResult::Success("First Task completed!".to_string())).unwrap();
            }
            Command::SecondTask => {
                println!("Processing SecondTask...");
                // Process the command and send the result
                result_sender.send(CommandResult::Success("Second Task completed!".to_string())).unwrap();
            }
        }
    };

    // Start the slot with the handler
    slot.start(handler);

    // Example of sending commands
    if let Err(e) = signal.send(Command::FirstTask) {
        eprintln!("Error sending command: {}", e);
    }
    if let Err(e) = signal.send(Command::SecondTask) {
        eprintln!("Error sending command: {}", e);
    }
}
```

## Contributing  
Contributions are welcome! Please fork the repository, create a feature branch, and submit a pull request.  

## License  
This project is licensed under the MIT License.  

## Contact  
For support or questions, open an issue or reach out on [GitHub Discussions](https://github.com/saturn77/egui_mobius/discussions).
