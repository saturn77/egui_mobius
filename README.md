
# Mobius_Egui  
Modular and ergonomic construction of egui applications. 

![Mobius_Egui Logo](./assets/mobius_strip.png)  

**Mobius_Egui** is a Rust framework designed to modularize and separate front-end and back-end logic in Egui applications. It emphasizes clean architecture with seamless communication between UI and business logic, inspired by the continuous, one-sided nature of the Möbius strip.  

## Why Mobius_Egui?  
In traditional Egui applications, UI and backend logic are often tightly coupled. Mobius_Egui solves this by providing a structured approach to communication between layers, improving maintainability and scalability.  

## Features  
- Clear separation of UI and business logic.  
- Flexible command and event processing using `std::mpsc`.  
- Modular design for cleaner, more maintainable code.  

## Quick Start
There are multiple crates in the repository, and each example
is a crate. 
One can test out an example by running these steps, in this
case the simple_monitor, and just subsitute an appropriate 
example name.  
```bash
git clone git@github.com:saturn77/mobius_egui.git 
cd mobius_egui
cargo build
cd examples
cargo run -p simple_monitor
```

## Installation
When using mobius_egui as a library, add the following to your `Cargo.toml`:  
```toml
[dependencies]
mobius_egui = "0.1.0"
egui = "0.31.0"
eframe = { version = "0.31.0", default-features = false, features = [
    "default_fonts", 
    "glow",          
    "wayland",       
] }
```  

## Usage  
Example of sending a command from the UI, inspired by the concept of Signals and Slots:
```rust
if ui.button("First Task").clicked() {
    Signal!(self.command_sender, Command::FirstTask);
}
```


Example backend processing:  
```rust
match command {
    Command::FirstTask => {
        logger_text.lock().unwrap().push_str("Processing FirstTask...\n");
        result_sender.send(CommandResult::Success("First Task completed!".to_string())).unwrap();
    }
}
```  

## Contributing  
Contributions are welcome! Please fork the repository, create a feature branch, and submit a pull request.  

## License  
This project is licensed under the MIT License.  

## Contact  
For support or questions, open an issue or reach out on [GitHub Discussions](https://github.com/saturn77/mobius_egui/discussions).  
