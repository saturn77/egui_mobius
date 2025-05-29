<div align="center">
<img width=260 height=200 src="https://raw.githubusercontent.com/saturn77/egui_mobius/master/assets/mobius_strip.png"></img>

# egui_mobius  
*Because GUI software design is a two sided problem operating on a single surface.*

[![egui_version](https://img.shields.io/badge/egui-0.31.1-blue)](https://github.com/emilk/egui)
[![egui_taffy](https://img.shields.io/badge/egui__taffy-0.7.0-purple)](https://github.com/Veykril/egui_taffy)
![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
[![Latest Version](https://img.shields.io/badge/version-0.3.0--alpha.31-green.svg)](https://crates.io/crates/egui_mobius)
[![Crates.io](https://img.shields.io/crates/v/egui_mobius.svg)](https://crates.io/crates/egui_mobius)
[![Rust](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml)
![Rust 2024](https://img.shields.io/badge/rust-2024-blue.svg)

</div>

egui_mobius is a comprehensive software stack for building sophisticated egui applications. It provides essential layers needed for production-ready GUI applications that include reactive state management, async operations and dedicated threading mechanisms, and an evolving component system. The component system is the primary
focus to develop modular and maintainble applications with a clean architecture.

## Core Features

The egui_mobius ecosystem provides multiple paradigms for building modern GUI applications, each serving different needs while maintaining compatibility with one another:

  ### Architecture Overview
  The diagram below is a general representation of how egui_mobius is organized. 

<div align="center">
<img width=360 height=330 src="./assets/mobius_stack.png"></img>
</div>

### Reactive State Management
- Thread-safe reactive primitives via `Dynamic<T>` and `Derived<T>` 
- Automatic UI updates when state changes
- Efficient dependency tracking with minimal boilerplate
- Composition-friendly design patterns with `ReactiveWidgetRef`

### Async Runtime
- Background processing that keeps your UI responsive
- Type-safe message passing between threads
- Built on Tokio for reliable async operations
- Seamless integration with the reactive system

### Modular Architecture
- Signal-slot system for clean separation of UI and business logic
- `MobiusWidget` traits for encapsulated, reusable UI elements
- Scalable patterns for complex applications
- Stateful components that maintain their own lifecycle

### Ecosystem

The egui_mobius framework consists of multiple coordinated crates:

- `egui_mobius`: Core signal-slot and dispatching system
- `egui_mobius_reactive`: Thread-safe reactive state management
- `egui_mobius_widgets`: Custom, stateful widget implementations
- `egui_mobius_components`: Higher-level UI components

## Built with egui_mobius

- `egui_lens` A reactive event logger [component](https://github.com/saturn77/egui_lens) built upon `egui_mobius_reactive`
- `diskforge` An SD card formatting [application](https://github.com/saturn77/egui_lens/tree/master/examples/diskforge) that is an example within `egui_lens`
- `KiForge` A computer aided manufacturing [CAM](https://github.com/saturn77/KiForge) platform for KiCad incorporating `egui_lens`

### Getting Started

Explore our comprehensive examples to understand different architectural approaches:

- `clock_reactive`: Modern reactive UI with minimal boilerplate
- `clock_async`: Thread-aware async operations with clean UI feedback
- `reactive_slider`: ReactiveWidgetRef for retained-mode style composition
- `logger_component`: EventLogger component for sophisticated event tracking

For the fastest start, check out our [template repository](https://github.com/saturn77/egui_mobius_template):

```bash
git clone https://github.com/saturn77/egui_mobius_template.git
cd egui_mobius_template
```

The template provides three comprehensive examples:

* **Reactive** - Basic reactive UI demo showing fundamental state management
* **Reactive-Async** - Sophisticated async task handling with background operations
* **Signals-Slots** - Full-featured RLC Circuit Simulator demonstrating signal-slot architecture

### Contributing  
* Contributions are welcome! Please fork the repository, create a feature branch, and submit a pull request.  
* This project is licensed under the MIT License.  
* For support or questions, open an issue or reach out on [GitHub Discussions](https://github.com/saturn77/egui_mobius/discussions).