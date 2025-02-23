# Mobius
Modular and ergonomic construction of egui applications. 

![alt text](assets/mobius_strip.png)

## Quick Start
One can test out the examples by running these steps 
```bash
git clone git@github.com:saturn77/mobius_egui.git 
cd mobius_egui
cargo build
cargo run --example simple_monitor
```

## Introduction 
The Egui framework is powerful and effective, but is challenging to 
work with in larger applications. Considering the many benefits of 
Egui, it is worthwhile to consider making a framework or library that
will modularize the code and partition the front and back ends to a 
degree that makes the code more : 

 - maintainable
 - versatile
 - composable

The implementation of these features relies on the concept of backend and frontend, and sender and receivers. Considering this, the name Mobius was adopted because eventually the front and back ends will function as a one side surface even though they are separated in code. 

## Planned Features

### Core Functionality
- Modularization system
- Async support via Tokio
- Exploration of Signals and Slots mechanism

### User Interface Components
- Common UI elements (buttons, sliders, text inputs, etc.) as *reusable* widgets
- Customizable themes based on existing Egui theming libraries

### State Management
- Global state management; which is the default for Egui
- Local state management

### Documentation and Examples
- Comprehensive documentation
- Example projects

### Testing and Debugging Tools
- Unit tests
- Debugging tools

### Performance Optimization
- Efficient rendering
- Resource management

