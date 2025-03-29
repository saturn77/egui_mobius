<div align="center">
<img width=260 height=200 src="https://raw.githubusercontent.com/saturn77/egui_mobius/master/assets/mobius_strip.png"></img>

# egui_mobius  
*Because GUI software design is a two sided problem operating on a single surface.*

[![egui_version](https://img.shields.io/badge/egui-0.31.1-blue)](https://github.com/emilk/egui)
[![egui_taffy](https://img.shields.io/badge/egui__taffy-0.7.0-purple)](https://github.com/Veykril/egui_taffy)
![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)
[![Latest Version](https://img.shields.io/badge/version-0.3.0--alpha.25-green.svg)](https://crates.io/crates/egui_mobius)
[![Crates.io](https://img.shields.io/crates/v/egui_mobius.svg)](https://crates.io/crates/egui_mobius)
[![Rust](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml/badge.svg?branch=master)](https://github.com/saturn77/egui_mobius/actions/workflows/rust.yml)
![Rust 2024](https://img.shields.io/badge/rust-2024-blue.svg)

</div>

egui_mobius is a layer 2 solution that transforms egui's immediate mode foundation into a complete application development platform. It enhances egui's efficient rendering with sophisticated state management, clean separation of UI and business logic, and a powerful signal-slot system. 

# Core Features

egui_mobius enhances egui with **multiple** powerful paradigms for building modern GUI applications. From reactive programming and async runtime capabilities to a thread-aware signal-slot architecture, each paradigm serves different needs and can be used independently or in combination. While the initial development of egui_mobius focused on the signal-slot architecture, the reactive and async paradigms have come to the forefront with the most recent releases of egui_mobius. 

## Reactive Programming
At its core, egui_mobius brings reactive programming to egui through its `Value<T>` and `Derived<T>` primitives. Developer ergonomics are enhanced with this architecture, allowing encapsulation of a reactive state management system. This is typical of a reactive styled component, where the main state application logic can simply update state and the component will respond. Thread-safe state management ensures your application remains responsive and data-consistent across UI and background operations.

## Async Runtime
Long-running tasks run in dedicated threads through a clean signal-slot communication system, keeping your UI responsive. The `MobiusRuntime` is built on Tokio, and provides robust handling of background operations while maintaining type safety and clear data flow between components. When combined with reactive programming, the async runtime provides a powerful framework for building responsive and responsive applications.

## Modular Architecture
The signal-slot system naturally encourages a clean separation between UI and business logic. Start with a simple frontend-backend split using type-safe messaging between modules. As your application grows, this pattern scales elegantly to support multiple specialized components while maintaining clear boundaries and interfaces.

## Production Ready
Built with real-world applications in mind, egui_mobius incorporates proven patterns from mature frameworks. Integration with Taffy brings powerful layout capabilities, enabling responsive designs with flexbox-style controls that adapt to your application's needs.


# Getting Started

There are numerous examples in this repo to provide a point of reference, and the recommended examples to explore are `clock_reactive`, `clock_async`, and `ui_refresh_events` to give a good overview of the different paradigms.

However, the **fastest way** to get started with egui_mobius is through our [template repository](https://github.com/saturn77/egui_mobius_template).

```bash
git clone https://github.com/saturn77/egui_mobius_template.git
cd egui_mobius_template
```

The template provides three comprehensive examples showcasing different architectural patterns:

* **Reactive** - Basic reactive UI demo showing fundamental state management
* **Reactive-Async** - Sophisticated async task handling with background operations
* **Signals-Slots** - Full-featured RLC Circuit Simulator demonstrating signal-slot architecture

Each example comes with detailed documentation and demonstrates best practices for building production-ready applications with egui_mobius.

## Version Status
egui_mobius follows semantic versioning and is currently in alpha (0.3.0-alpha.25). While in alpha, we maintain API stability within minor versions. See [VERSIONING.md](VERSIONING.md) for details.

## Contributing  
* Contributions are welcome! Please fork the repository, create a feature branch, and submit a pull request.  


* This project is licensed under the MIT License.  

* For support or questions, open an issue or reach out on [GitHub Discussions](https://github.com/saturn77/egui_mobius/discussions).
