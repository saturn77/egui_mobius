# egui_mobius  
*Because GUI software design is a two sided problem operating on a single surface.*


![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
![Latest Version](https://img.shields.io/badge/version-0.2.0-green.svg)
![Rust](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml/badge.svg?branch=main&event=push)


![egui_mobius Logo](./assets/mobius_strip.png)  

**egui_mobius** is a Rust framework to modularize and separate front-end and back-end logic in Egui applications, with a top level feature of future proofing the core of a design. It emphasizes clean architecture with seamless communication between UI and business logic, inspired by the continuous, one-sided nature of the Möbius strip. It is meant to be part of the egui ecosystem of helper crates, but also has other possible front ends, as the crux crate is progressively introduced into the library.  

## Motivation 
In traditional Egui applications, UI and backend logic are often tightly coupled. egui_mobius solves this by providing a structured approach to communication between layers, improving maintainability and scalability. Having core elements of a gui that are portable and maintainble is the ultimate goal.  

## Features  
- Clear separation of UI and business logic.  
- Flexible command and event processing using `std::mpsc`.
- Employ the signals and slots paradigm.   
- Modular design for cleaner, more maintainable code. 
- Portable containers via crux for core backend logic.  

## Quick Start
There are multiple crates in the repository, and each example
is a crate. 
One can test out an example by running these steps, in this
case the simple_monitor, and just substitute an appropriate 
example name.  
```bash
git clone git@github.com:saturn77/egui_mobius.git 
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
use egui_mobius::{factory, Command, CommandResult};

fn main() {
    // Create a signal-slot pair using the factory method
    let (signal, slot) = factory::<Command, CommandResult>();

    // Define a handler function for the slot
    let handler = |command: Command| {
        match command {
            Command::FirstTask => {
                slot.send(CommandResult::Success("First Task completed!".to_string())).unwrap();
            }
            Command::SecondTask => {
                slot.send(CommandResult::Success("Second Task completed!".to_string())).unwrap();
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
